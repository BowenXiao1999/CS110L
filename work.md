# Typing System Problems

**assigned to [Ricky Xu](https://singularity-data.quip.com/IHAAEAzr4EY)**

## Context

In RisingWave, we plan to support the following types in MVP: `boolean`, `smallint`, `integer`, `bigint`, `float4`, `float8`, `numeric`, `time`, `date`, `timestamp`, and `timestampz`.

Currently, we do not support `timestampz` and `numeric`.

A data value can be represented in different types throughout the query life cycle. Consider the following queries:

```
create table t(v1 timestamp, v2 timestamp);
insert into t values('2020-01-01', '2021-01-01');
select * from t;
```

In this query, `'2020-01-01'` and `'2021-01-01'` are both parsed as string type in the parser. The binder checks the corresponding column type and then convert the string type to timestamp type, which is internally stored as `int64_t`. When fetching the data from the database, we should convert `int64_t` into string to return to the user.

The typing system is required to cast the data into proper types and determine whether overflow can occur. For example, if executing the following queries:

```
create table t(v1 smallint, v2 smallint);
insert into t values(11111111111, 11111111111);
```

Postgres will directly report out-of-range error.

Sometimes, implicit type casting is required to be done internally in the system. For example, in the following queries:

```
create table t(v1 smallint, v2 int);
insert into t values(1, 1);
select v1 + v2 from t;
```

The database will automatically cast `v1` from `smallint` type to `integer` type at binding phase before processing the `+` operator at execution time.

The database shall also perform out-of-range detection during the execution time. For example, in the following queries:

```
create table t(v1 smallint, v2 smallint);
insert into t values(22222, 22222);
select v1 + v2 from t;
```

Postgres directly return out-of-range error. To achieve this, we should promote both side (v1 and v2) from smallint to bigint and perform plus operator.

When both v1 and v2 are (positive) bigint, `v1 + v2 > INT64_MAX` is unsafe but `v1 > INT64_MAX - v2` is okay. Similar tricks can be used on additions on negative bigint as well as subtraction and multiplication. ([postgres src](https://git.postgresql.org/gitweb/?p=postgresql.git;a=blame;f=src/include/common/int.h;h=079954d7f0b13cf116a4a59cedceb99383d855f6;hb=HEAD)) [Mingji Han](https://singularity-data.quip.com/MYaAEAHbkTs)



## Specs

Some of these requirements could be done concurrently (which are grouped into same stage). 
Specs have different priority levels. Each level roughly equals to: 

* P0: Must support for MVP 
* P1: Good to have for MVP, might not be necessary. 
* P2: Definitely not crucial, but could be future hackathon/rampup/interview tasks


**Stage 0:** 

* [P0] Support implicit type casting for numerical types arithmetic operations. (We will try to support permutation of multiple numerical types, mixing them together) 

```
create table t(v1 smallint, v2 smallint);
insert into t values(22222, 22222);
select v1 + v2 from t; // implicate type casting 
```

* [P0] Raise exceptions for type overflows 

```
create table t(v1 smallint, v2 smallint);
insert into t values(11111111111, 11111111111); // error
 
insert into t values(22222, 22222);
select v1 + v2 from t; // error
```

* [P0] Support creation of tables with type `time`, `date`,  `timestamp`, `timestampz` and `numeric` including `numeric(precesion, scale)` . For details of the numeric type, refer to postgres doc [here](https://www.postgresql.org/docs/current/datatype-numeric.html). 

    * [Ricky]: If we don’t have international use cases (cross timezones), we could probably de-prioritize support for `timestampz` ?

```
create table t(v1 timestamp);
create table t(v1 time);
create table t(v1 date);
create table t(v1 timestampz);
create table t(v1 numeric);
create table t(v1 numeric(10, 5));
```

* * *
**Stage 1:** 

* [P0] Support implicit rounding on `numeric` types and exception for numeric overflow 

```
create table t(v1 numeric(3, 1));
insert into t values(10.1); // OK
insert into t values(10.15); // OK, but rounds to 10.2 
insert into t values(-10.15); // OK, but rounds (away from 0) to -10.2
insert into t values(99.94); // OK
insert into t values(99.95); // raise errors 
```

* [P0] Support date time related insertion ([refer to this section for all possible format](https://www.postgresql.org/docs/current/datatype-datetime.html)). Depends on the ease of implementing multiple variants for each type, we might choose to support some of them, not the others. But having at least one format for each type is the minimal requirement. 

```
insert into t values('2021-01-02'); // ISO 8601 format for date 
insert into t values('`04:05:06'``);`` ``// ISO 8601 for time, there are a few differnt `
insert into t values('1999-01-08 04:05:06'); // ISO 8601 for timestamp 
insert into t values('1999-01-08 04:05:06 -8:00'); // for timestampz 
```

* * *
**Stage 2:**

* [P1] Select ISO/Postgres/SQL style output for data/time. [Ricky Xu](https://singularity-data.quip.com/IHAAEAzr4EY) thinks supporting one of them is enough. 

```
`SQL style``:`` ``12``/``17``/``1997`` ``07``:``37``:``16.00`` PST`
`ISO ``:`` ``1997``-``12``-``17`` ``07``:``37``:``16``-``08`
`Postgres``:`` Wed Dec 17 07:37:16 1997 PST`
```

* [P1] Support special values for data/time related types: ([ref 8.5.1.4](https://www.postgresql.org/docs/current/datatype-datetime.html))
    * `now`
    * `today`
    * `tomorrow`
    * `yesterday` 
* [P1] Support date/time operations. We could pick some of those only involve `time`, `date`, and `timestamp` from the [operations table(Table 9.31)](https://www.postgresql.org/docs/current/functions-datetime.html)
* [P1] Support `Nan` values for `numeric` type. For specs of `nan` value, refer to the postgres numeric doc [section](https://www.postgresql.org/docs/current/datatype-numeric.html)

```
create table t (v1 numeric);
insert into t values('nan');
```

* [P2] Support data time functions. some could be trivial to suuport e.g. `current_date`, `current_time`, `date_part`, ... Some might be hard, for which we could cherrypick 

## Type Inference system

To support implicit casting, the binder infer types of expressions during the binding phase. Each `AbstractExpression` has a `DataValueType`. When evaluating the expression, such expression should return a vector of given type. 


|	|Children	|Output	|
|---	|---	|---	|
|Constant	|-	|	|
|TypeCast	|any	|casted type	|
|Column_ref	|-	|column type	|
|Compare	|two	|bool	|
|Subquery	|?	|?	|
|Aggregate	|any non-boolean	|See below	|
|Conjunction	|two	|bool	|
|Operator	|two	|the 'bigger' type between two sides	|

**Type cast table**

|From\To	|bool	|smallint	|integer	|bigint	|float4	|float8	|numeric	|time	|date	|timestamp	|timestamptz	|
|---	|---	|---	|---	|---	|---	|---	|---	|---	|---	|---	|---	|
|bool	|-	|✕	| ✓	|✕	|✕	|✕	|✕	|✕	|✕	|✕	|✕	|
|smallint	|✕	|-	| ✓	| ✓	| ✓	| ✓	| ✓	|✕	|✕	|✕	|✕	|
|integer	| ✓	| ✓	|-	| ✓	| ✓	| ✓	| ✓	|✕	|✕	|✕	|✕	|
|bigint	|✕	| ✓	| ✓	|-	| ✓	| ✓	| ✓	|✕	|✕	|✕	|✕	|
|float4	|✕	| ✓	| ✓	| ✓	|-	| ✓	| ✓	|✕	|✕	|✕	|✕	|
|float8	|✕	| ✓	| ✓	| ✓	| ✓	|-	| ✓	|✕	|✕	|✕	|✕	|
|numeric	|✕	| ✓	| ✓	| ✓	| ✓	| ✓	|-	|✕	|✕	|✕	|✕	|
|time	|✕	|✕	|✕	|✕	|✕	|✕	|✕	|-	|✕	|✕	|✕	|
|date	|✕	|✕	|✕	|✕	|✕	|✕	|✕	|✕	|-	| ✓	| ✓	|
|timestamp	|✕	|✕	|✕	|✕	|✕	|✕	|✕	| ✓	| ✓	|-	| ✓	|
|timestamptz	|✕	|✕	|✕	|✕	|✕	|✕	|✕	| ✓	| ✓	| ✓	|-	|

**Aggregation return type**

|Function	|Input type(s)	|Return type	|
|---	|---	|---	|
|Sum	|smallint, integer, bigint, float4, float8, numeric	|bigint for smallint or int; numeric for bigint; same for float4, float8, numeric	|
|Count	|any	|bigint	|
|Avg	|smallint, integer, bigint, float4, float8, numeric	|numeric for smallint, int, bigint; float8 for float4 float8; numeric for numeric;	|
|Max	|smallint, integer, bigint, float4, float8, numeric, date, time, timestamp, timestamptz	|same as input	|
|Min	|smallint, integer, bigint, float4, float8, numeric, date, time, timestamp, timestamptz	|same as input	|

TODO:

* Type cast from boolean in integer
* Type cast between `time`, `date`, `timestamp` and `timestamptz`
* Detect erroneous types fed in operator expression 
* select 1+true from table;
* Detect erroneous types fed in conjunction expression, e.g. 
* select V from table where T>1 and 1
* Detect erroneous types fed in comparison expression, e.g. 
* select V from table wher T>true



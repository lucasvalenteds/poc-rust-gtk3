drop database if exists rust;
create database rust;
use rust;

create table ACCOUNT (
    CODE char(36),
    REMOVED tinyint(1)
);

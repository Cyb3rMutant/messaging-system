#!/bin/sh

./wait-for db:3306

cargo run

#!/bin/bash

# install all available nightly versions for a given month
month=12
for ((day=15; day<=31; day++))
do
  rustup toolchain install nightly-2021-$month-$day
done

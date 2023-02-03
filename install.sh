#!/bin/bash

set -e

mkdir -pv /usr/share/blimp
cp -v target/release/blimp /usr/bin/

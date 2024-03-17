#!/bin/bash

while true; do
  /usr/bin/vector &>> /var/log/vector.log
  echo "Vector exited with status $?"
  sleep 60
done

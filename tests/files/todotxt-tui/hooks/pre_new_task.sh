#!/bin/bash

if [ "$1" == "UPDATE" ] ; then
  echo "This task has been updated by hook."
else
  echo "$1"
fi


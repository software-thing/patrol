#!/bin/sh

echo "Applying migrations"
dbmate -v
dbmate -url "sqlite:data/patrol.db" up

echo "Starting Patrol"
exec patrol

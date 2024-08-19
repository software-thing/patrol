#!/bin/sh

echo "Applying migrations"
dbmate -v
dbmate up

echo "Starting Patrol"
exec patrol

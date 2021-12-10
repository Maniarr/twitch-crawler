#!/bin/bash

export WARP10_WRITE_TOKEN=$(cat $WARP10_WRITE_TOKEN_FILE)
/usr/bin/twitch-crawler

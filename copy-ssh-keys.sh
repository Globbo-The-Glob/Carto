#!/bin/bash

USER=pi

echo "RPI $PI_IP"

ssh-copy-id $USER@$PI_IP

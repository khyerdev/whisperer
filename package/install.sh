#!/usr/bin/bash

sudo mkdir /usr/local/bin/whisperer
sudo mv ./tcp.png /usr/local/bin/whisperer/tcp.png
sudo mv ./whisperer /usr/local/bin/whisperer/whisperer
mv ./whisperer.desktop $HOME/.local/share/applications/whisperer.desktop

echo Successfully moved files
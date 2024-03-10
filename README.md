<h1><img src="https://github.com/OSCH2008/whisperer/blob/master/package/tcp.png" style="height: 1em;"> Whisperer <img src="https://github.com/OSCH2008/whisperer/blob/master/package/tcp.png" style="height: 1em;"></h1>
Whisperer is a simple app made in rust (btw) to directly send encrypted messages between computers on your local network without a middleman. Simply open the app on both computers you want to send messages between, get the local IP address of one computer, plug it in on the other, and start sending encrypted messages!

<img src="https://github.com/OSCH2008/whisperer/blob/master/assets/github/superior.png" style="display: inline-block; width: 50%;"><img src="https://github.com/OSCH2008/whisperer/blob/master/assets/github/inferior.png" style="display: inline-block; width: 50%;">

The messages are encrypted and decrypted using a simple and non-standard algorithm. It should be enough considering you are only communicating on your local network. Who is gonna attempt to intercept LAN traffic anyway?

---

## Features
Whilst being a tiny application made by a 15-year old, Whisperer has more than 2 unique features:

1. Whisperer uses the TCP protocol for sending and receiving messages. This means you can send and receive messages between different operating systems.
2. Obviously, you can send messages. The character limit for each message is 2000 characters. You are able to press ENTER anywhere in the app to send the message you have currently typed. Messages will not send if the receiving computer does not have the app open.
3. The protocol I made for sending/receiving messages uses the concept of symmetric encryption for encrypting and decrypting messages.
4. The recipients, private keys, and chat histories are stored on your system when you close the app, and are restored when you open it back up.
5. When adding new recipients, the app will check if the IP entered is a valid IP by sending a specific byte to that machine, and expecing another specific byte to be returned back. It also checks if the recipient is already added. When adding a new recipient, a chat history and private key will be set up on your machine, and on the other end.
6. You can set, change, and remove aliases for recipients on your end. If a recipient has an assiged alias, the alias will show in the chat history window instead of their IP. In the recipient select bar, the alias will be shown before the IP, with the IP surrounded in parentheses. Aliases have a length limit of 28 characters, so that the longest IP address (in terms of characters) with a 28 character alias can still fit in the minimum (and default) size of the window.
7. You have the option to locally clear the chat history of the current recipient. You also have the option to completely remove the recipient, chat history, and private key. You do have to click the button for it twice, so you dont accidentally remove it. If the recipient you removed attempts to send a message to you, a new keypair will be built and the recipient will be automatically set back up on your machine.
8. You do not get notifications for any incoming messages, neither in the app nor when it is closed.

## Installation
### Linux
- In order to run whisperer, your GLIBC version needs to be 2.35 or higher. You can check your current GLIBC version by running `ldd --version` in your terminal of choice. I have also made a [tiny script](https://github.com/OSCH2008/dotfiles/blob/main/.scripts/glibc-reqver) to get the required GLIBC version of any executable.
- If your GLIBC is up-to-date, the installation process is very easy:

  1. On the [main github page of this repo](https://github.com/OSCH2008/whisperer), navigate to the [releases page](https://github.com/OSCH2008/whisperer/releases) or click the latest version under the releases section.
  2. Find and click the download link for `whisperer.tar.gz`.
  3. In your terminal of choice or file manager of choice, navigate to your browser's download directory.
  4. Extract the file you downloaded. You can use a GUI based file extractor, or you can use the terminal by running `mkdir whisperer; tar -xzf whisperer.tar.gz -C whisperer; cd whisperer`.
  5. Run `sudo ./install.sh` to move the files to the correct spots. Ignore any errors yapping about "file already exits". You can either move uninstall.sh to a memorable directory, or you can delete it.
  6. You can now run whisperer through your desktop environment's application manager or by running `whisperer` in the terminal.

- If your GLIBC is not up-to-date but your operating system's package repo does not have a newer version, you can compile the binary yourself on your machine. To do this, you need to install `git`, `rust`, and `gcc`, and follow these steps in your terminal of choice:

  1. Clone the repository by running `git clone https://github.com/OSCH2008/whisperer.git; cd whisperer`.
  2. Compile the binary by running `cargo build --release`.
  3. Overwrite the incompatible binary by running `mv -f target/release/whisperer package/whisperer`.
  4. `cd package` into the package folder, then start at step 5 of the installation process used for an up-to-date glibc.

### Windows
- On Windows 10/11, the app is guarunteed to work out-of-the-box. The installation process is simple:

  1. On the [main github page of this repo](https://github.com/OSCH2008/whisperer), navigate to the [releases page](https://github.com/OSCH2008/whisperer/releases) or click the latest version under the releases section.
  2. Find and click the download link for `whisperer.exe`.
  3. The app will work as intended simply by running it. You can optionally move it to another folder, pin it to your start menu/taskbar, and/or make a shortcut for it on the desktop.

Whisperer is currently not supported on MacOS and will not be for the forseeable future. I do not own any apple products and I do not plan on owning any. Plus, running a Mac VM is next to impossible for me. You can try to compile it yourself on mac, but I have only set up conditional compiling for Windows and Linux.

## Dependencies
Whisperer uses the following crates to make implementing what I wanted to implement significantly easier:

- [rand](https://crates.io/crates/rand): For generating random bytes used for the ciphers
- [eframe](https://crates.io/crates/eframe): My personal favorite native GUI library for rust. Extremely easy to use, and filled with many features.
- [once_cell](https://crates.io/crates/once_cell): Used for lazily assigning non-static values to a mutable static variable in a safe way.
- [image](https://crates.io/crates/image): Used for decoding the ICO file format and getting the raw RGBA from an icon file embedded in the binary.

None of these crates are used to cheat around the actual logic of the program, they ore only used to make things atleast work without days, weeks, or months of research and crying.

## License
This project is licensed under the [GNU GPL v3](https://www.gnu.org/licenses/gpl-3.0.en.html), meaning you are allowed to freely use, modify, and distribute this project as long as you keep it as free and open-source as this project is, and as long as you include the same lisence and indicate your changes. More information about this license is [here (fossa.com)](https://fossa.com/blog/open-source-software-licenses-101-gpl-v3/) and [here (gnu.org)](https://www.gnu.org/licenses/quick-guide-gplv3.html).

I would prefer if you credited my work when you share this.

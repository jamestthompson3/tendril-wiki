# Tendril Wiki

[![Build And Test](https://github.com/jamestthompson3/platform/actions/workflows/rust.yml/badge.svg)](https://github.com/jamestthompson3/platform/actions/workflows/rust.yml)

## Installation

### From source

This requires you to have the Rust toolchain installed. You can install this by cloning the repo, and running `cargo install --path .`. Then run `tendril -i` to bring up the interactive bootstrapper.

### From latest release

Download the latest binary for your OS from the releases page. After unzipping the folder, run `./tendril -i` to bootstrap your wiki. You will then want to add this to your path with `sudo mv ./tendril /usr/local/bin` or annotating you `.bashrc` to point to the location where you've unzipped the release folder.

## Getting started

Before starting, you'll need to run `tendril -i` to bootstrap your wiki. An important note here is that when asked for a password, you are not encrypting the notebook, but rather it acts as a password for the webserver. Unauthorized requests will be rejected, but the notebook itself will still be stored in plaintext files on disk.

### Running the wiki

After bootstrapping the wiki, you can run `tendril` to start the webserver.

### Building a static site

You can also build a static site by runing `tendril -b`.


### Customization

You can find your configuration and your custom CSS files in the config directory. The location of this directory depends on your platform and will be printed out when you run `tendril --version`.

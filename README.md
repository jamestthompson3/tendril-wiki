# Tendril Wiki

[![Build And Test](https://github.com/jamestthompson3/platform/actions/workflows/rust.yml/badge.svg)](https://github.com/jamestthompson3/platform/actions/workflows/rust.yml)

![Tag Page](assets/screenshot1.png)
![Note](assets/screenshot2.png)

### Nice features
- Works without JavaScript
- Self hosted, files can be composed easily with other tools since they are plaintext
- Both light and dark themes

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

### Updating your installation

You can make sure that you copy over any new template or config files after each update by running `tendril -u` after downloading the latest release or building from source.


### Customization

You can find your configuration and your custom CSS files in the config directory. The location of this directory depends on your platform and will be printed out when you run `tendril --version`.

You can also set a per-note favicon by uploading the image you wish to use as a favicon and then adding the "icon" field to the notes metadata:

```md
---
title: magic and computers
icon: potion.svg
tags: [emergence, technology]
---
```

This will look for the `potion.svg` file uploaded by you and set it as the favicon for the `magic and computers` note.

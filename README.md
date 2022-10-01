# Tendril Wiki

[![Build And Test](https://github.com/jamestthompson3/platform/actions/workflows/rust.yml/badge.svg)](https://github.com/jamestthompson3/platform/actions/workflows/rust.yml)

## Contents

- [Installation](#Installation)
- [Migration](#migration)
- [Getting started](#getting-started)

![Landing Page](assets/home_page.png)

### Nice features

- Self hosted, files can be composed easily with other tools since they are plaintext
- Both light and dark themes
- Can build your notebook as a static site

![Tag Page](assets/screenshot1.png)
![Note](assets/screenshot2.png)

## Installation

### From source

This requires you to have the Rust toolchain installed. You can install this by cloning the repo, and running `cargo install --path .` from inside the `bin` directory. Then run `tendril -i` to bring up the interactive bootstrapper.

### From latest release

Download the latest binary for your OS from the releases page. After unzipping the folder, run `./tendril -i` to bootstrap your wiki. You will then want to add this to your path with `sudo mv ./tendril /usr/local/bin` or annotating you `.bashrc` to point to the location where you've unzipped the release folder.

## Migration

If you have an existing Tendril Wiki installation that is pre v1.0.0, you will need to do the following steps to migrate it to the latest version:

- Stop your currently running Tendril Wiki server.
  - Note: You might want to temporarily disable git sync if you have it enabled. This will allow you to check the migration without pushing the changes to your git repository.
  - v1 of Tendril Wiki uses wikitext instead of markdown and its initial release doesn't support the full markdown spec, so there might be broken formatting. If you still want full markdown support, do not migrate.
- Add one addtional field to the `general` section of your config file. This field should be `check_for_updates` and its value is either `true` or `false`. This value determines whether or not the client will show a message when there is a new release of Tendril Wiki.
- Run the migrate command. This will create a backup of your current wiki directory in the same parent directory as your wiki. This means if your wiki is located in `~/Documents/wiki`, the migration tool will create `~/Documents/tendril-backup`. The command for running the migration tool is `tendril -m`.
- Restart your Tendril Wiki server and check the changes.
- If the changes are acceptable, you can re-enable git sync if you turned it off, and restart your server to allow this change to take effect.

## Getting started

Before starting, you'll need to run `tendril -i` to bootstrap your wiki. An important note here is that when asked for a password, you are not encrypting the notebook, but rather it acts as a password for the webserver. Unauthorized requests will be rejected, but the notebook itself will still be stored in plaintext files on disk.

### Running the wiki

After bootstrapping the wiki, you can run `tendril` to start the webserver.

You can also run tendril wiki as a service on your operating system. Inside the `services`
directory, you'll find template files for both Linux and MacOS (PRs for Windows support are welcome
:^) ). This will allow to configure tendril wiki to automatically start when you log into your
computer.

### Building a static site

You can also build a static site by runing `tendril -b`.

### Updating your installation

You can make sure that you copy over any new template or config files after each update by running `tendril -u` after downloading the latest release or building from source.

### Interstitial Journaling

You can use Tendril Wiki for interstitial journaling both through the command line or through the web interface! From
the command line, you can call `tendril`, followed by the line you want to add to the daily log:

```bash
tendril "I'm working on adding features I want for my workflow in Tendril Wiki"
```

This will append the line to the daily log entry, or create the entry if it does not exist. It's important to note that
you will need to properly escape quotes from you shell if you choose to update the daily log via the CLI. After
processing the entry, Tendril Wiki will also automatically update the git repo if you've chosen to use the git sync
feature in your config file.

### Bookmark Archiving

If you tag a note with `bookmark`, and add `url:<your-url-here>`, to the metadata editor, tendril
will automatically archive the full text of the URL. This text will subsequently be available to be
searched by tendril's search engine, allowing you to run a full text search on your bookmarks.

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

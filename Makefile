linux-x86:
	cross build --release --target x86_64-unknown-linux-musl
arm:
	cross build --release --target arm-unknown-linux-gnueabihf
win:
	cross build --release --target x86_64-pc-windows-msvc
all: linux-x86 arm win

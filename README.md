# cyberpunk_mod_manager

Simple mod manager for cyberpunk

# ###WARNING

* It might corrupt your cyberpunk install
* It won't work with mods that replace certain files as it will delete the replaced file instead of restoring it to its original state

# How it works

The mod manager only tries to check if the mod zip file downloaded from (Preferably nexus mods) still has its files in your cyberpunk directory, if not it can move them there to install it for you or remove them by comparing the file structure of the original mod zip file to the files in the cyberpunk install dir

# Why did I make this

I am constantly bricking my Cyberpunk install by trying out new mods and I just wanted a quick and easy way to test mods


# Future Goals

This project is just a quick and dirty utility, i am not going to maintain it regularly (might do for major bugs), contributions are welcome

# Screenshots
![cyberpunk_mod_manager](https://user-images.githubusercontent.com/66156000/206888889-d92e3fc5-1cb0-4606-af34-b08ad1f6accb.png)

# Compatability
This program is only tested on Windows 11
libarchive is required to be installed to run this program, you can install it using vcpkg from [here](https://github.com/microsoft/vcpkg) 
I had to use the triplet x64-windows-static-md to get it to work
sample command:
```
./vcpkg.exe install libarchive --triplet=x64-windows-static-md
```

# Credits
Huge thanks to [Nexus Mods](https://www.nexusmods.com/) for providing a great modding community and a great modding platform

Thank you [fdehau](https://crates.io/crates/tui) for creating tui-rs it is an amazing tool for creating terminal applications
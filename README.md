# Sunset Shimmer

Shim-tools for Windows

Current:

In *nix you can create a script in any language, just putting a hashbang line, and give it execution permissions chmod +x and you can use said script as a executable in any application.

It's very useful to create alias, set default environments, etc.

In windows... we don't have that luck.

Even if you can create a directory and add the extension to the env var PATHEXT, this is only respected by a shell or Run as..., but a "normal" program trying to execute will look for an exe file.

Putting every binary file path into PATH is also not good, because there is not only binary files but also library files that can .. the PATH.

Symlinks doesn't work because Win programs tend to look for their libraries in the same directory the binary is in, so if you symlink, the exe will look for the library in the symlink path.

Sunset helps you create "shims", proxy executables that execute other exes and proxy their I/O and signals.

The shim executable reads a local file (named the same the executable, but with shim extension).

Sunset is named on a popular character of a equestrian cartoon and toy line, as it is a "Shimmer"

## Configuration

By default, shims are created in the `%LOCALAPPDATA%\sunset\shims` path.



Create a file `%LOCALAPPDATA%/sunset/config.toml`

```toml
shim_path = "C:\\Users\\<username>\\AppData\\Local\\sunset\\shims"
```

## Usage




Create a shim that executes a specific EXE

sunset shim PATH_TO_EXE

If PATH_TO_EXE is a single exe name, it will be looked on the PATH

sunset path

sunset rm

sunset upgrade

sunset list

sunset upgrade-all

## Prior art

Shim tool from Scoop

https://github.com/ScoopInstaller/Shim


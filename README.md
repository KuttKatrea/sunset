# Sunset Shimmer: Shim-tools for Windows

## Rationale

Consider an *nix environment.

You have installed an application by extracting files in some path, (e.g. `/opt/myapp/`, where the actual binary is `/opt/myapp/bin/myapp`).

You want to be able to call that binary from anywhere (a terminal or any other applications), so you need to have that binary accessible in the `PATH`.

For that, you can use one of the following options:

- Add the `bin` application directory to the PATH with:

	```sh
	export PATH=/opt/myapp/bin/:$PATH
	```

- Create a symbolic link in an existing directory already in the `PATH` (e.g. `/usr/local/bin`). The created symbolic link could be considered a *shim*.

	```sh
	ln -s /opt/myapp/bin/myapp /usr/local/bin/myapp
	```

	(This may not work if myapp requires some resources relative to the executable, and it doesn't verify if it's a symlink)

- Create a shell script in a directory already included in the `PATH` (e.g. `/usr/local/bin/`) with the following content:

	```shell
	#!/us/bin/sh
	exec /opt/myapp/bin/myapp "$@"
	```

	And give it execution permissions:

	```shell
	chmod +x /usr/local/bin/myapp`
	```

The last option is a *launcher script* that will replace the script call with the myapp execution mantaining the reference to the real path it lives on, preventing the second option possible issues, and the execution being transparent to any caller.

This technique can be used also to create aliases with predefined arguments to the `myapp` call with the content:

```shell
#!/us/bin/sh
exec /opt/myapp/bin/myapp --some-parameter "$@"
```

Or even to set some environment variables by default:

```shell
#!/us/bin/sh
export MYAPP_ENV_VAR="some-value"
exec /opt/myapp/bin/myapp --some-parameter "$@"
```

### Windows alternatives

If we are working in a Windows environment, under similar circunstances where we have an application files extracted in a directory (e.g. `C:\Apps\myapp\` with the executable actually being `C:\Apps\myapp\myapp.exe`), and we want myapp.exe to be able to be invoked in any other program, we can try the same options than in *unix:

- Add the the `C:\Apps\myapp\` directory to the `PATH`

	```bat
	set PATH=C:\Apps\myapp\;%PATH%
	```

	The downside of this option, is that in windows is common to have libraries (DLLs) along the executable, so by adding the `C:\Apps\myapp\` directory to the `PATH`, we may also be adding all the dll libraries and other executables to the `PATH` as well, leaving it dirty.

- Create a directory for binaries, add it to the `PATH` and create symbolic links in that directory.

	```bat
	mkdir C:\bin
	set PATH=C:\bin;%PATH%
	mklink C:\bin\myapp C:\Apps\myapp\myapp.exe
	```

	The donside of this alternative, is the same as the second alternative in *nix: if the application tries to load resources from the directory the application is in, it may try to load them from `C:\bin` instead of `C:\Apps\myapp\`. In Windows this is more common than in *nix. Also, In Windows you need Administrator rights to be able to create symbolic links.

- Create a launcher script.
	
	```bat
	@echo off
	start "" "C:\Apps\myapp\myapp.exe" %*
	```

	This is the more common alternative used in Windows, similar to the third option for *nix, and traditionally using batch files (`.bat`) but also possible to use other types like Powershell scripts (`.ps1`) or even python ones (`.py`).

	Windows has an environment variable `PATHEXT` that defines which extensions are considered "programs" that can be run from a terminal, or "Run as..." commands (more specifically, that can be executed by the ShellExec call).

	But, even if these scripts can be called by other programs by using the ShellExecute system call, they don't behave themselves the same way as in *unix: the process is not replaced, `myapp` is launched as another process, and the launcher script just ends. Also, if the caller application doesn't use ShellExec but CreateProcess instead, they can't be called (CreateProcess requires a real executable).

	So, for maximum compatibility, in Windows it's better to have **real** executables instead of *launcher scripts*.

## Enter Sunset

Sunset helps you create *shims*.

*Shims* are executables that proxy execution, I/O and signals to another executable, waiting for it to finish execution and returning the same return code.

With this behaviour, it mimics more precisely what a launcher script in *nix does.

The *shims* that Sunset creates, reads a file named the same as the shim, but with the `.shim` extension and executes the target executable based on the description on that file.

The *shim descriptor file* looks like:

~~~toml
path = 'C:\Apps\myapp\myapp.exe'
args = [
	'arg1',
	'arg2'
]
~~~

## Installation

Extract the executables `sunset.exe`, `shim.exe` and `shimw.exe` to a directory already on the `PATH`. (These executables don't have dependencies).

Execute `sunset init` to configure your environment.

This will create the `%LOCALAPPDATA%\sunset\shims` directory, and will add it to the `PATH`.

The shims will be created in this directory.

## Usage

Executing

```bat
sunset shim C:\Apps\myapp\myapp.exe
```

Will create a shim in `%LOCALAPPDATA%\sunset\shims\myapp.exe` that will execute `C:\Apps\myapp\myapp.exe` by loading the descriptor at `%LOCALAPPDATA%\sunset\shims\myapp.shim`.

## Prior art

Other known shim tools, that doesnt support advanced features as environment variables, or GUI applications.

- https://github.com/ScoopInstaller/Shim
- https://github.com/71/scoop-better-shimexe/tree/master

## Naming

Sunset is named on a popular character of a equestrian cartoon and toy line.

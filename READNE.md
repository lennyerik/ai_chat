# ai_chat
Do you love free features like [DuckDuckGo's AI Chat](https://duckduckgo.com/aichat), but wish you could use them in your terminal as well?
Well, wish no more, this tool allows you to ask your ML model of choice any question you like directly on the command line. With an alias or symlink set up to `ai_chat`, your workflow could look something like this:

````
$ ls
Cargo.lock  Cargo.toml  LICENSE.txt  READNE.md  src  target

$ ai "how do i make the output of ls pretty?"
To make the output of the `ls` command more visually appealing, you can use various options and flags. Here are some common ways to achieve a "pretty" output:

1. **Colorized Output**: Most modern terminal emulators support colorized output. You can enable it by using the `--color` option:
   ```bash
   ls --color=auto
   ```

2. **Long Format**: To get more detailed information about files (like permissions, owner, size, and modification date), use the `-l` option:
   ```bash
   ls -l --color=auto
   ```

3. **Human-Readable Sizes**: If you want file sizes to be displayed in a more understandable format (e.g., KB, MB), you can add the `-h` option:
   ```bash
   ls -lh --color=auto
   ```

4. **Sorting Options**: You can sort files by modification time, size, or name. For example, to sort by modification time, use:
   ```bash
   ls -lt --color=auto
   ```

5. **Grouping by Extension**: To group files by their extensions, you can use the `--group-directories-first` option:
   ```bash
   ls -l --group-directories-first --color=auto
   ```

6. **Combining Options**: You can combine these options for a more comprehensive output. For example:
   ```bash
   ls -lh --color=auto --group-directories-first
   ```

7. **Using Aliases**: To make it easier to use these options, you can create an alias in your shell configuration file (like `.bashrc` or `.zshrc`):
   ```bash
   alias ll='ls -lh --color=auto --group-directories-first'
   ```

After adding the alias, you can simply type `ll` to get the pretty output.

By using these options, you can customize the output of `ls` to make it more informative and visually appealing.

$ ls -l --color=auto
total 40
-rw-r--r-- 1 lenny lenny 19984 Nov 18 11:41 Cargo.lock
-rw-r--r-- 1 lenny lenny   305 Nov 18 11:41 Cargo.toml
-rw-r--r-- 1 lenny lenny  1067 Nov  7 18:33 LICENSE.txt
-rw-r--r-- 1 lenny lenny   378 Nov 18 11:39 READNE.md
drwxr-xr-x 2 lenny lenny  4096 Nov 18 11:40 src
drwxr-xr-x 3 lenny lenny  4096 Nov 18 11:41 target

$ # :)
````

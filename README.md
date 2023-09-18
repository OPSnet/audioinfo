# AudioInfo Generator
The AudioInfo Generator is a command-line tool that generates an audioinfo file for a given directory containing FLAC files. This README provides an overview of the tool's functionality, usage, and command-line options.

# Usage
```audioinfo-generator [FLAGS] --input <DIRECTORY> [--output <OUTPUT>] [--print]```

*  `--input: Sets the directory to scan for FLAC files (required).`

*  `--output: Sets the output directory for the generated audioinfo file. If not provided, the audioinfo file will be saved in the current working directory with the name "audioinfo.txt".`

*  `--print: If specified, the generated audioinfo content will be printed to the standard output instead of saving it to a file.`

* `--verbose or -v: Enables verbose (debug) output. This option can be used to get more detailed information during the execution of the tool.`
* `-h: Show help menu of the tool`

# Trailing slashes
Currently clap has a issue with trailing slashes on paths. So if you encounter a issue when trying to generate a file with a path such as
```.\audioinfo.exe inputi 'H:\test\test\test_album\'```
Please remove the trailing slash as this causes clap to attach a ``"`` to the end
# Examples
## Generate an audioinfo file for a directory 
```.\audioinfo.exe -i 'H:\test\test\test_album'
## Generate an audioinfo file for FLAC files in a specified directory and save it to a custom output file:
```audioinfo --input /path/to/your/flac_files --output /path/to/output/audioinfo.txt```
## Generate an audioinfo file for FLAC files in a specified directory and print the content to the standard output:
```audioinfo-generator --input /path/to/your/flac_files --print```
## Enable verbose (debug) output:
```audioinfo --input /path/to/your/flac_files --verbose```

# AudioInfo Generator
The AudioInfo Generator is a command-line tool that generates an audioinfo file for a given directory containing FLAC files. This README provides an overview of the tool's functionality, usage, and command-line options.

# Usage

```audioinfo-generator [FLAGS] --directory <DIRECTORY> [--output <OUTPUT>] [--print]```

*  `--directory: Sets the directory to scan for FLAC files (required).`

*  `--output: Sets the output directory for the generated audioinfo file. If not provided, the audioinfo file will be saved in the current working directory with the name "audioinfo.txt".`

*  `--print: If specified, the generated audioinfo content will be printed to the standard output instead of saving it to a file.`

* `--verbose or -v: Enables verbose (debug) output. This option can be used to get more detailed information during the execution of the tool.`


# Examples
## Generate an audioinfo file for FLAC files in a specified directory and save it to a custom output file:
```audioinfo --directory /path/to/your/flac_files --output /path/to/output/audioinfo.txt```
## Generate an audioinfo file for FLAC files in a specified directory and print the content to the standard output:
```audioinfo-generator --directory /path/to/your/flac_files --print```
## Enable verbose (debug) output:
```audioinfo --directory /path/to/your/flac_files --verbose```
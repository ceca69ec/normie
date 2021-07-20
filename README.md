normie
======

**Just another filename normalizer tool.**

Recursively normalize directories and filenames to Unix friendly standard.

No dependencies, really simple and fast.

## Example

```console
$ touch "B)E(T%T@E*R T*H*I&S W@A*Y#" "G)O(O%@D N*A*M&E@**#"
$ ls
'B)E(T%T@E*R T*H*I&S W@A*Y#'  'G)O(O%@D N*A*M&E@**#'
$ normie -lra .tgz *
$ ls
better_this_way.tgz  good_name.tgz
```

## Help

```shell
normie 1.0.0

USAGE:
    normie [FLAG]... DIRECTORY_OR_FILE...

FLAGS:
    -a: Append the specified text at the end of the filename.
    -h: Show this help information.
    -i: Insert the specified text at the beginning of the filename.
    -l: Transform the resulting filename into all lowercase characters.
    -r: Remove these characters: '!"#$%&'()*+,/:;<=>?@[\]^`{|}~ªº'.
    -t: Interactively asks for confirmation of each action.
    -u: Transform the resulting filename into all uppercase characters.
    -v: Show information about the performed actions.
```

## Warning

Use flag `-t` if you are insecure of the results.

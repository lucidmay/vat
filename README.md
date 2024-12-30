# Vat

Vat is a cross-platform tool for managing software packages. Inspired by [Rez](https://github.com/AcademySoftwareFoundation/rez), but with a focus on simplicity and ease of use.


The package versions are managed using `git tag`.


## The Basics
### Vat Commands
- `vat init` - Initialize a new package in the current directory.
- `vat repo-init` - Initialize a new repository in the current directory.
- `vat repo` - Prints vat's repository path.
- `vat up` - Update the package to the latest version.
- `vat publish --message <message>` - Publish the package to the repository.
- `vat test <command> --append <package_name>...` - Run the tests for the current package.
- `vat read` - Read the current package.


### Vat Environment Commands
Vat environment commands are used to manage and interact with the packages published in the vat repository.
- `vat-env <package_name> <command> --append <package_name>...` - Run the command for the given package.
- `vat-env --list` - List all the packages in the repository.
- `vat-env <package_name> --list-cmds` - List all the commands for the given package.
- `vat-env --remove <package_name>...` - Remove the given packages from the repository.


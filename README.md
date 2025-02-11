# Vat

Vat is a simple lightweight cross-platform tool for managing software packages and resolving environment variables. Inspired by [Rez](https://github.com/AcademySoftwareFoundation/rez), but with a focus on simplicity and ease of use.



The package versions are managed using `git tag`.


## The Basics
### Vat Commands
- `vat init` - Initialize a new Vat package in the current directory.
- `vat new <package_name>` - Create a new Vat package.
- `vat up` - Update the package to the latest version.
- `vat publish --message <message>` - Publish the package to the repository.
- `vat run <command> --package <package_name> --append <package_name> --detach` - Run the command for the given package.
  - `<command>` Run the command for the given package.
  - `--package <package_name>` Run the command for the given package. If not provided, it will use the package in the current directory. If the current directory is not a package, it will try to resolve the package from the repository.
  - `--append <package_name>` Append the given packages to the environment variables.
  - `--detach` Spawn the command in a new process.
- `vat link` - Link the package to the repository, without publishing it.
- `vat cat` - Print the package information.
- `vat repo` - Print packages in the repository.

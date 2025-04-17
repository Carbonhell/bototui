# Bototui

Bototui is an opinionated TUI for AWS. The objectives of this crate are multiple:
- offer a frontend for AWS that can be used by users who may not have a good understanding of how AWS works, but who need to interface with specific services for day-to-day operations;
- offer an opinionated alternative with simpler UX flows for certain services, such as AWS Batch, which feel a bit painful to use.

This project was generated from the [component template][CTemplate] described in the [Ratatui][Ratatui] docs.

## Other info

- The default actions shipped with the template used are currently disabled and will be enabled when the project is fleshed out.

## Very rough roadmap

- [x] Integrate basic usages of the AWS SDK from an architectural PoV - e.g. spawning tokio tasks to perform the necessary info lookups etc
- [x] Make a very simple homepage listing the services currently supported by Bototui
- [x] Start working on the AWS Batch component, with the objective of listing jobs only
- [x] Complete the AWS batch component by integrating the required calls to AWS to retrieve jobs
- [ ] Allow terminating a Batch job
- [ ] Figure out how to do proper testing, and if possible reuse this strategy to allow running a simple demo without requiring real AWS resources
- [ ] Unify how keybindings are done (they're still hardcoded in some places, such as handle_key_event in home.rs)


## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
[CTemplate]: https://ratatui.rs/templates/component
[Ratatui]: https://ratatui.rs

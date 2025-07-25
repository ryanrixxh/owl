# owl

An AWS stack visualiser and configuration helper. Made using [Ratatui].

[Ratatui]: https://ratatui.rs
[Simple Template]: https://github.com/ratatui/templates/tree/main/simple

# TODOS / Features
- List a given stack in the current CDK deployment 
    - List all resources in a deployment. Stateful list, where each entry can be selected and selecting it will navigate to the AWS console.
    - Additional: List all environments and compare differences? I dont think this is possible because the SDK will use different profiles. Maybe if we are able to read the credentials file and then grab stacks for all of the profiles we could do it?
- List and update SSM configurations in AWS for a given environment
- Change a given integration from maintenance mode to production mode
- CHange


## License

Copyright (c) ryanrixxh <ryanmbr@protonmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE

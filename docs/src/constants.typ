#let repo-url = "https://github.com/oicana/oicana"
#let latest-cli = "https://github.com/oicana/oicana/releases/tag/oicana_cli-v0.1.0-alpha.3"
#let latest-cli-shell = ```bash

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/oicana/oicana/releases/download/oicana_cli-v0.1.0-alpha.3/oicana_cli-installer.sh | sh
```
#let latest-cli-powershell = ```psh

-ExecutionPolicy Bypass -c "irm https://github.com/oicana/oicana/releases/download/oicana_cli-v0.1.0-alpha.3/oicana_cli-installer.ps1 | iex"
```

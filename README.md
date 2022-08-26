`tombl` 
=======

`tombl` makes `bash` viable for DevOps-automations that involve configurations
saved as `.toml` files.

It allows `bash` to read `.toml` files **structurally**, so you don't have to
come up with weird ad-hoc solutions involving `awk`, `sed`, and tears as soon as
it breaks in production because you didn't use an actual toml-parser.

```bash
$ set -euo pipefail
$ tombl -e DB=databases.hmm /etc/my-config.toml
declare -A DB=(["user"]="postgreker" ["password"]="super secret" ["host"]="0.0.0.0" ["port"]=5432)
$ eval "$(tombl -e DB=databases.hmm /etc/my-config.toml)"
$ echo "${DB[user]}"
postgreker
$ pg_dumpall -h "${DB[host]}" -p "${DB[port]}" -u "${DB[user]}" > out.sql
```

Bash is unable to store nested arrays of any kind, so any nesting will be
ignored when exporting, and you'll have to adapt your `-e VAR=path.to.thing` to
access the nested information. It is recommended that you start your scripts with
`set -euo pipefail`

```bash
$ set -euo pipefail
$ cat /etc/my-config.toml
[databases.hmm]
user = "postgreker"
password = "super secret"
host = "0.0.0.0"
port = 5432
thing-that-is-nested = { will-not-be-included = 123 }
$ tombl -e DB=databases.hmm /etc/my-config.toml
declare -A DB=(["user"]="postgreker" ["password"]="super secret" ["host"]="0.0.0.0" ["port"]=5432)
$ eval "$(tombl -e DB=databases.hmm /etc/my-config.toml)"
$ echo "${DB[thing-that-is-nested]}" # whoops, but this will fail fast because of `set -euo`
bash: l: unbound variable
```

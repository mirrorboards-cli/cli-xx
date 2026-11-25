# cli-xx

`xx`: następca `x`. Właściciel określa jego polecenia jedno po drugim, korzystając z niego na co
dzień ([`cli-spec/xx.typ`](../cli-spec/xx.typ)). `x` działa obok bez zmian.

Oba programy stoją na tym samym rdzeniu (`cli-runtime`, biblioteka `xkeys-cli`, `xkeys-session`,
biblioteki wtyczek) i widzą ten sam stan: plik `~/.config/mirror/config.toml`, konteksty, skarbce,
sesje i zmienne `MIRROR_*`. `xx` nie uruchamia wtyczek `x-*`.

Grupa poleceń narzędzia leży w crate tego narzędzia, w module `xx` (`xx vault` w
`xkeys-cli/src/xx/`), obok wiersza poleceń `x`; ten crate tylko składa grupy. Wycofanie `x` usunie
jego wiersze poleceń, a moduły `xx` zostaną.

## Uruchomienie

```sh
cargo build                 # w tym katalogu; program trafia do wspólnego target/debug
xx --help
cargo install --path .      # albo xx w ~/.cargo/bin
```

## Polecenia

| Polecenie | Co robi |
| --- | --- |
| `xx vault create <NAME>` | zakłada skarbiec, z kopią do folderu przez `--backup` (`xkeys-cli/src/xx/vault/create.rs`, `backup.rs`) |
| `xx vault create --wizard` | to samo, pytanie po pytaniu (`xkeys-cli/src/xx/vault/create_wizard.rs`) |
| `xx vault unlock <NAME>... \| --all` | otwiera skarbiec, sesja trzyma go otwartym (`xkeys-cli/src/xx/vault/unlock.rs`) |
| `xx vault lock <NAME>... \| --all` | blokuje skarbiec teraz (`xkeys-cli/src/xx/vault/lock.rs`) |
| `xx vault sync <NAME> [--to <URL>]` | synchronizuje teraz; `--to file://…` zaczyna synchronizację w folderze (`xkeys-cli/src/xx/vault/sync.rs`) |
| `xx identity create <VAULT>/<NAME>` | tworzy tożsamość w odblokowanym skarbcu (`xkeys-cli/src/xx/identity/create.rs`) |
| `xx stacks init <PATH> --identity <VAULT>/<IDENTITY>` | zakłada albo sprawdza hasło platformy (`dev-cli-xstacks/src/xx/stacks/init.rs`) |
| `xx stacks deploy <PATH>` | wdraża stack, grupę albo platformę (`dev-cli-xstacks/src/xx/stacks/deploy.rs`) |
| `xx stacks preview <PATH>` | pokazuje, co zmieniłby deploy (`dev-cli-xstacks/src/xx/stacks/preview.rs`) |
| `xx vault import <FILE> [NAME] [--sync]` | sprowadza skarbiec z pliku `.xkeys`; `--sync` synchronizuje w nim dalej (`xkeys-cli/src/xx/vault/import.rs`) |
| `xx vault guard list\|portable\|local …` | strażnicy skarbca (`xkeys-cli/src/xx/vault/guard.rs`) |
| `xx vault list [NAME]...` | tabela skarbców jak `kubectl get`; `-o wide|name|json|table` (`xkeys-cli/src/xx/vault/list.rs`) |
| `xx version` | wypisuje wersję `xx` |

## Nowe polecenie

1. Właściciel opisuje polecenie; jego postać (wywołanie, flagi, pomoc) jest pokazana do akceptacji.
2. Polecenie grupy narzędzia: `<crate narzędzia>/src/xx/<grupa>/<polecenie>.rs`, grupa podpięta
   w `src/command_line.rs`; polecenie samego `xx`: `src/commands/<polecenie>.rs`.
3. Logika poza warstwą poleceń leży we wspólnym rdzeniu; logika ukryta w poleceniu `x` przechodzi
   do biblioteki, z której korzystają oba programy, bez zmiany zachowania `x`.
4. Testy, wpis w tabeli wyżej i w `cli-spec/xx.typ`.

Pomoc i komunikaty po angielsku. Błąd to jedna linia `xx: …` i kod 1, błędnie wpisane polecenie
kod 2.

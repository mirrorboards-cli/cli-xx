# cli-xx

`xx`: następca `x`. Właściciel określa jego polecenia jedno po drugim, korzystając z niego na co
dzień ([`cli-spec/xx.typ`](../cli-spec/xx.typ)). `x` działa obok bez zmian.

Oba programy stoją na tym samym rdzeniu (`cli-runtime`, biblioteka `xkeys-cli`, `xkeys-session`,
biblioteki wtyczek) i widzą ten sam stan: plik `~/.config/mirror/config.toml`, konteksty, skarbce,
sesje i zmienne `MIRROR_*`. `xx` nie uruchamia wtyczek `x-*`.

Grupa poleceń narzędzia leży w crate tego narzędzia, w module `xx` (`xx vault` w
`xkeys-cli/src/xx/`), obok wiersza poleceń `x`; ten crate tylko składa grupy. Wycofanie `x` usunie
jego wiersze poleceń, a moduły `xx` zostaną.

## Instalacja

Najnowsze wydanie z GitHuba (macOS na Apple Silicon, macOS 15 lub nowszy):

```sh
curl -fsSL https://raw.githubusercontent.com/mirrorboards-cli/cli-xx/main/install.sh | sh
xx update                   # później: najnowsze wydanie w miejsce tego programu
```

`install.sh` sprawdza sha256 archiwum i kładzie `xx` w `/usr/local/bin`, a gdy nie można tam
pisać, w `~/.local/bin` (inny katalog: `XX_INSTALL_DIR`). Wydania to tagi `v<wersja>` z
`xx-macos-arm64.tar.gz` i `xx-macos-arm64.tar.gz.sha256`; wersja jest w `Cargo.toml`.

## Wydanie

1. Wpisy z **Unreleased** w `CHANGELOG.md` przechodzą pod nową wersję, a `Cargo.toml` dostaje
   tę wersję. Przypięcia w `.github-ci/mirror.toml` idą na bieżący `main` zależności (CI buduje
   z przypiętych commitów). Wszystko wypchnięte na `main`.
2. `xx release publish v0.3.0 --identity xx-releases/releaser` robi resztę:
   - uruchamia workflow **Release** (GitHub Actions: `mctl sync` przypiętych zależności z sekretem
     `MCTL_SSH_KEY`, `scripts/build-release-macos.sh` na macOS) i czeka na szkic wydania
     z archiwum, `.sha256` i opisem z changelogu, około 15 minut;
   - pobiera archiwum, sprawdza sha256, podpisuje je tożsamością (zablokowany skarbiec otwiera
     się na to jedno polecenie), sprawdza podpis kluczami `RELEASE_KEYS` jak `xx update`;
   - wgrywa `.sig` i publikuje wydanie; z `--draft` zostawia szkic do obejrzenia.
   Gdy szkic z archiwum już istnieje (np. po przerwanym podpisie), polecenie go nie buduje
   ponownie. Potrzebuje zalogowanego `gh`.

## Zmiany

Każda zmiana `xx` trafia od razu do [`CHANGELOG.md`](CHANGELOG.md) pod **Unreleased**; wydanie
przenosi te wpisy pod swoją wersję, a ta sekcja jest opisem wydania na GitHubie.

## Uruchomienie ze źródeł

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
| `xx git upload <ROOM> <FILE>... [--to <DIR>]` | pliki do repozytorium pokoju w XGit jednym commitem, bez klonu (`xgit-cli/src/xx/git/upload.rs`) |
| `xx git files <ROOM> [PATH]` | pliki gałęzi z rozmiarami, które trzyma XGit (`xgit-cli/src/xx/git/files.rs`) |
| `xx git clone <ROOM> [DIR]` | klon pokoju dla zwykłego gita, z `git config xx.identity` (`xgit-cli/src/xx/git/clone.rs`) |
| `xx git install` | `git-remote-xgit` jako link do `xx` i filtr dużych plików w globalnym git config (`xgit-cli/src/xx/git/install.rs`) |
| `xx git retain [REMOTE]`, `xx git objects fetch [PATH]...` | w klonie: żywe obiekty treści, treść zamiast wskaźników (`xgit-cli/src/xx/git/in_repository.rs`) |
| `xx sql query <ROOM> <SQL> [-o table\|json\|csv]` | wiersze zapytania do bazy pokoju (`xsql-cli/src/xx/sql/query.rs`) |
| `xx sql exec <ROOM> [SQL]... [--file <FILE>\|-]` | instrukcje po kolei, zmiany idą do pokoju jednym rekordem (`xsql-cli/src/xx/sql/exec.rs`) |
| `xx sql export <ROOM> <FILE> [--follow]` | odszyfrowana baza pokoju w pliku SQLite (`xsql-cli/src/xx/sql/export.rs`) |
| `xx sql snapshot <ROOM>` | migawka bazy dla nowych członków (`xsql-cli/src/xx/sql/snapshot.rs`) |
| `xx stacks init <PATH> --identity <VAULT>/<IDENTITY>` | zakłada albo sprawdza hasło platformy (`dev-cli-xstacks/src/xx/stacks/init.rs`) |
| `xx stacks deploy <PATH>` | wdraża stack, grupę albo platformę (`dev-cli-xstacks/src/xx/stacks/deploy.rs`) |
| `xx stacks preview <PATH>` | pokazuje, co zmieniłby deploy (`dev-cli-xstacks/src/xx/stacks/preview.rs`) |
| `xx vault import <FILE> [NAME] [--sync]` | sprowadza skarbiec z pliku `.xkeys`; `--sync` synchronizuje w nim dalej (`xkeys-cli/src/xx/vault/import.rs`) |
| `xx vault remove <NAME>` | usuwa skarbiec z tego komputera (`xkeys-cli/src/xx/vault/remove.rs`) |
| `xx vault guard list\|portable\|local …` | strażnicy skarbca (`xkeys-cli/src/xx/vault/guard.rs`) |
| `xx vault list [NAME]...` | tabela skarbców jak `kubectl get`; `-o wide|name|json|table` (`xkeys-cli/src/xx/vault/list.rs`) |
| `xx wallet get_global_properties` | parametry łańcucha Mirrornet, metoda `cli_wallet` (`mirrornet-cli/src/xx/wallet/get_global_properties.rs`) |
| `xx wallet get_dynamic_global_properties [--watch]` | bieżący stan łańcucha; `--watch` wiersz na blok (`mirrornet-cli/src/xx/wallet/get_dynamic_global_properties.rs`) |
| `xx wallet import_balance <ACCOUNT> --identity <VAULT>/<IDENTITY> [--preview]` | odbiera salda z genesis spod kluczy tożsamości na konto (`mirrornet-cli/src/xx/wallet/import_balance.rs`) |
| `xx wallet create_account <VAULT>/<NAME> --registrar <VAULT>/<IDENTITY>\|<URL> [--fund <AMOUNT>]` | konto tożsamości: płaci tożsamość-rejestrator albo rejestrator XSystemu (`mirrornet-cli/src/xx/wallet/create_account.rs`) |
| `xx wallet create_company_account <NAME> --founder <VAULT>/<IDENTITY> --member <ACCOUNT>=<WEIGHT>... --threshold <N> --registrar <URL>` | konto firmy przez rejestratora XSystemu; owner i active z tymi samymi wagami (`mirrornet-cli/src/xx/wallet/create_company_account.rs`) |
| `xx mirrorboard deploy [PATH] --identity <VAULT>/<IDENTITY>` | infra wdraża instancję mirrorboarda; kod wyjścia 3, gdy czekają podpisy (`mirrornet-cli/src/xx/mirrorboard/deploy.rs`) |
| `xx mirrorboard sign <HOST>/<CODE>` | właściciel podpisuje u siebie to, na co czeka wdrożenie (`mirrornet-cli/src/xx/mirrorboard/sign.rs`) |
| `xx mirrorboard github [PATH] --identity <VAULT>/<IDENTITY> --registrar <URL>` | ustawia instancję do wdrażania z GitHub Actions: skarbiec i konto CI, slot hasła stosu, dwa sekrety i workflow wspólny z `mirrorboards/mirrorboard-deploy`, w pull requeście (`mirrornet-cli/src/xx/mirrorboard/github_setup.rs`) |
| `xx release publish <TAG> --identity <VAULT>/<IDENTITY> [--draft]` | buduje w CI, podpisuje i publikuje wydanie `xx` (`src/commands/release.rs`) |
| `xx update [--check] [--version <TAG>]` | podmienia `xx` na najnowsze wydanie z GitHuba (`src/commands/update.rs`) |
| `xx completion fish\|zsh\|bash` | skrypt dopełniania po Tab: polecenia, flagi, skarbce, tożsamości, sieci (`src/completion.rs`); `install.sh` zakłada go sam |
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

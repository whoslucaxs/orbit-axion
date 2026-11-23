# Desenvolvimento e distribuição

## Compilar e executar

Requer Windows 11 x64, Rust MSVC 1.95+, Build Tools C++ e Windows SDK.

```powershell
cargo build --locked --release -p orbit-axion --bins
.\target\release\orbit-axion.exe
```

Para forçar renderização software, defina `SLINT_BACKEND=winit-software` antes de executar. Normalmente o backend seleciona GPU com fallback disponível. O overlay mantém o centro no cursor, inclusive próximo às bordas: pode ficar parcialmente cortado.

## Verificar

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
cargo run --locked -p orbit-axion --bin orbit-probe
cargo run --locked -p orbit-axion --bin orbit-axion -- --smoke-test
cargo run --locked -p orbit-axion --bin orbit-axion -- --ui-test
```

`orbit-probe` testa instalação/remoção de hooks sem interceptar e lê os provedores de áudio. `--smoke-test` verifica overlay real, foco, estilos e geometria; `--ui-test` verifica persistência dos valores ao criar a página de preferências e, na compilação debug, exercita o controlador com cinco botões, cliques atrasados e modo clique/segurar entre eventos reais da interface, sem instalar hooks nem injetar entrada. Ambos usam armazenamento isolado em target. Veja [TEST_PLAN.md](TEST_PLAN.md) para verificações de hardware ainda necessárias.

## Instalador e atualizações

```powershell
python packaging/windows-msi/collect_notices.py
$orbitVersion = python -c "import tomllib; print(tomllib.load(open('Cargo.toml', 'rb'))['workspace']['package']['version'])"
python packaging/windows-msi/build_msi.py target/release/orbit-axion.exe "dist/OrbitAxion-$orbitVersion-x64.msi" $orbitVersion
```

Empacotamento nativo MSI requer Python 3.9–3.12 no Windows (`msilib`). Inclui escolha por usuário/máquina, upgrade transacional e remoção opcional dos dados no desinstalador interativo. Instalação silenciosa preserva dados. O build extrai o CAB e compara os hashes dos arquivos incorporados; instalação, upgrade e rollback precisam ser exercitados em VM antes de distribuição ampla.

O updater consulta releases GitHub do repositório configurado, compara versões, verifica SHA-256 fornecido pelo GitHub e pede consentimento para instalar. Para repositório privado, `ORBIT_GITHUB_TOKEN` precisa permitir leitura de releases. O token não é salvo no JSON. Sem release publicada, o app informa indisponibilidade; não há atualização fictícia.

## Código

| Diretório | Responsabilidade |
| --- | --- |
| crates/core | Configuração, validação, perfis, busca e geometria |
| crates/windows-platform | Hooks, DPI, foco, overlay, serviços Win32 |
| crates/audio | Core Audio e Voicemeeter Remote API |
| crates/app | Editor, execução, workers, armazenamento, updater e bandeja |
| ui | Interface Slint e tema |
| packaging | MSI e inventário de dependências |

A [especificação original](SPEC.md) é referência de produto, não um registro de testes aprovados. [FEASIBILITY.md](FEASIBILITY.md) registra somente o laboratório 0.1.0 e foi superado por esta implementação. Consulte [THIRD_PARTY.md](../THIRD_PARTY.md) antes de redistribuir.

Limites de validação: jogos, processos elevados, monitores com DPI diferente, carga prolongada e acessibilidade exigem testes dedicados. Composição IME não é suportada na busca sem foco; nessa situação, use seleção pelo mouse. Nenhum comando destrutivo de sistema foi executado durante os testes de desenvolvimento.

## GitHub Actions

O workflow **Rust Windows** verifica formatação, Clippy, testes, executáveis e o conteúdo do MSI em Windows. Alterações somente em README, CHANGELOG e docs não disparam nova compilação.

O upload de pacotes é independente da validação. Para disponibilizar EXE/MSI como artifacts, abra **Actions → Rust Windows → Run workflow** e habilite `upload_artifacts`. Os arquivos ficam disponíveis por três dias. Esse envio depende da cota de armazenamento da conta GitHub; sem cota, a etapa de upload falha explicitamente. Compilação, testes e empacotamento continuam sendo verificados nas execuções automáticas.

A versão do instalador é lida de `Cargo.toml`; o resumo da execução contém os hashes SHA-256 dos pacotes gerados. A execução com upload habilitado inclui EXE, MSI, inventário de dependências e `SHA256SUMS.txt`.

## Releases automáticas

Altere `workspace.package.version` em `Cargo.toml`, atualize `Cargo.lock` e o changelog e envie as mudanças à `main`. Após passar em todas as etapas do workflow, a automação cria a tag `v<versão>` e publica a release com EXE, MSI, `SHA256SUMS.txt`, avisos e licença. Não é necessário criar a tag nem subir arquivos manualmente.

A publicação usa o `GITHUB_TOKEN` do workflow, com permissão `contents: write` e credenciais disponibilizadas somente à etapa de publicação. Os arquivos são enviados diretamente para Releases, sem depender do armazenamento de artifacts do Actions. Pull requests não publicam releases.

Se a versão já estiver publicada, a etapa termina preservando seus arquivos. Um draft interrompido só é retomado quando pertence ao mesmo commit; versões de pré-lançamento em Cargo são marcadas como prerelease e não substituem a versão estável mais recente. A execução manual na main também pode concluir a publicação de uma versão pendente.

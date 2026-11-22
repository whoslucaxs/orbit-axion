# Orbit Axion

Menu radial para Windows 11 x64 em Rust + Slint, independente de periféricos Logitech. GPL-3.0-only.

## Versão 0.2.0

Editor de oito posições, submenus, perfis por caminho de executável com prioridade e herança do Global, modo manual, busca fuzzy, macros canceláveis, atalhos, texto Unicode, abertura de programas/scripts, mídia e comandos de janela/sistema. O anel abre somente por Meio, Mouse 4 ou Mouse 5; funciona por clique ou segurar/soltar, sem tomar o foco. Fechar o editor mantém o app na bandeja; use Sair na bandeja para encerrar.

Tema claro/escuro seguindo o sistema, paleta Inked (#000000, #DFDEDC, #A6A7A2, #464545, #00ACAC), superfícies translúcidas e brilho discreto. Tamanho, opacidade, cor de destaque, ícones, sons e idioma português/inglês configuráveis.

Áudio Windows por saída padrão ou endpoint específico; Voicemeeter Potato com oito strips e cinco buses. A roda ajusta somente sobre o ícone de volume; soltar após rolar não executa outra ação. Provedor indisponível mostra erro, sem trocar silenciosamente para outro. Potato deve estar instalado e aberto: a DLL proprietária não é distribuída.

Configuração local em `%LOCALAPPDATA%/OrbitAxion`: JSON validado, gravação atômica, até 30 backups e recuperação de arquivo inválido. Importação/exportação completa ou por perfil incorpora ícones. Sem telemetria, nuvem ou histórico de ações. Inicialização com Windows é opcional. O modo `--watchdog` reinicia falhas e oferece modo seguro; `--safe-mode` abre sem interceptação de entrada.

## Compilar e executar

Requer Windows 11 x64, Rust MSVC 1.95+, Build Tools C++ e Windows SDK.

```powershell
cargo build --locked --release -p orbit-axion --bins
.	argetelease\orbit-axion.exe
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

`orbit-probe` testa instalação/remoção de hooks sem interceptar e lê os provedores de áudio. `--smoke-test` verifica overlay real, foco, estilos e geometria; `--ui-test` verifica persistência dos valores ao criar a página de preferências. Ambos usam armazenamento isolado em target. Veja [docs/TEST_PLAN.md](docs/TEST_PLAN.md) para verificações de hardware ainda necessárias.

## Instalador e atualizações

```powershell
python packaging/windows-msi/collect_notices.py
python packaging/windows-msi/build_msi.py target/release/orbit-axion.exe dist/OrbitAxion-0.2.0-x64.msi 0.2.0
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

A [especificação original](docs/SPEC.md) é referência de produto, não um registro de testes aprovados. [FEASIBILITY.md](docs/FEASIBILITY.md) registra somente o laboratório 0.1.0 e foi superado por esta implementação. Consulte [THIRD_PARTY.md](THIRD_PARTY.md) antes de redistribuir.

Limites de validação: jogos, processos elevados, monitores com DPI diferente, carga prolongada e acessibilidade exigem testes dedicados. Composição IME não é suportada na busca sem foco; nessa situação, use seleção pelo mouse. Nenhum comando destrutivo de sistema foi executado durante os testes de desenvolvimento.

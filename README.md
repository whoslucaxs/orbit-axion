# Orbit Axion

Menu radial para Windows 11 x64 em Rust + Slint, independente de periféricos Logitech. GPL-3.0-only.

O binário MSVC requer o Microsoft Visual C++ Runtime x64 (VCRUNTIME140.dll); o MSI atual não instala esse pré-requisito.

## Versão 0.2.5

Editor de oito posições, submenus, perfis por caminho de executável com prioridade e herança do Global, modo manual, busca fuzzy, macros canceláveis, atalhos, texto Unicode, abertura de programas/scripts, mídia e comandos de janela/sistema. O anel abre por Meio, Mouse 4, Mouse 5, Esquerdo, Direito ou uma tecla do teclado; funciona por clique ou segurar/soltar, sem tomar o foco. Fechar o editor mantém o app na bandeja; use Sair na bandeja para encerrar.

Tema claro/escuro seguindo o sistema, paleta Inked (#000000, #DFDEDC, #A6A7A2, #464545, #00ACAC), superfícies translúcidas e brilho discreto. Tamanho, opacidade do fundo (0–100%), cor de fundo HEX, cor de destaque, ícones, sons e idioma português/inglês configuráveis. Cor de fundo vazia segue o tema do aplicativo. A página **Cores do menu** controla uma paleta independente: fundo, botões, ícones, texto, texto secundário, bordas, título, seleção e sombra, além da opacidade. As cores são escolhidas pelo seletor visual nativo do Windows, com controles de opacidade na interface. O botão **Salvar aparência do menu** salva somente a aparência do anel. Configurações antigas são migradas uma única vez para preservar a aparência anterior. Esquerdo/direito continuam livres dentro do editor para permitir configuração. Botões extras do fabricante precisam ser mapeados pelo driver para um botão padrão reconhecido pelo Windows.

Áudio Windows por saída padrão ou endpoint específico; Voicemeeter Potato com oito strips e cinco buses. A roda ajusta somente sobre o ícone de volume; soltar após rolar não executa outra ação. Provedor indisponível mostra erro, sem trocar silenciosamente para outro. Potato deve estar instalado e aberto: a DLL proprietária não é distribuída.

Configuração local em `%LOCALAPPDATA%/OrbitAxion`: JSON validado, gravação atômica, até 30 backups e recuperação de arquivo inválido. Importação/exportação completa ou por perfil incorpora ícones. Sem telemetria, nuvem ou histórico de ações. Inicialização com Windows é opcional. O modo `--watchdog` reinicia falhas e oferece modo seguro; `--safe-mode` abre sem interceptação de entrada.

## Personalização visual

- **Cores do menu:** fundo, botões, ícones, textos, bordas, seleção, sombra e círculo central. O círculo tem cor de fundo, borda e raio próprios (24–76 pixels na escala base do anel).
- **Menus e ações:** cores individuais de fundo, borda e ícone para cada botão; Padrão volta a herdar a paleta do menu.
- **Biblioteca de ícones:** 2.078 Bootstrap Icons e 2.234 Material Icons, com busca, filtro e paginação, incorporados ao executável para uso offline. Ícones de arquivo continuam disponíveis.
- **Liquid Glass:** opções independentes para o app e para o menu. É uma adaptação visual em Slint, com transparência, relevo e reflexos; não implementa as APIs Apple de refração de fundo. O fundo externo do anel permanece transparente.
- **Submenus em modo segurar:** pause 300 ms sobre um submenu para entrar mantendo o acionador pressionado. Soltar encerra o gesto, inclusive sobre o centro ou posições vazias. O modo clique mantém navegação persistente.

## Configurar o acionamento

Em **Preferências do app → Abrir menu com → Tecla do teclado**, escolha a tecla na lista e clique em **Aplicar preferências**. F13 é a seleção inicial, e a lista funciona mesmo quando a captura global está pausada. Também é possível usar **Capturar botão do mouse ou tecla (10 s)** e pressionar a tecla/botão desejado. O app identifica a origem e salva a escolha. Também é possível escolher Mouse/Teclado manualmente e aplicar preferências. A captura é de uma tecla individual (letras, funções F1–F24, navegação e modificadores), sem combinação de teclas. A tecla fica livre dentro do editor para permitir digitação; teste a abertura em outro aplicativo. Repetições de uma tecla mantida pressionada não reabrem o anel. Eventos enviados pelas próprias ações do Orbit são identificados para evitar recaptura.

## Compilar e executar

Requer Windows 11 x64, Rust MSVC 1.95+, Build Tools C++ e Windows SDK.

```powershell
cargo build --locked --release -p orbit-axion --bins
.	arget
elease\orbit-axion.exe
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

`orbit-probe` testa instalação/remoção de hooks sem interceptar e lê os provedores de áudio. `--smoke-test` verifica overlay real, foco, estilos e geometria; `--ui-test` verifica persistência dos valores ao criar a página de preferências e, na compilação debug, exercita o controlador com cinco botões, cliques atrasados e modo clique/segurar entre eventos reais da interface, sem instalar hooks nem injetar entrada. Ambos usam armazenamento isolado em target. Veja [docs/TEST_PLAN.md](docs/TEST_PLAN.md) para verificações de hardware ainda necessárias.

## Instalador e atualizações

```powershell
python packaging/windows-msi/collect_notices.py
python packaging/windows-msi/build_msi.py target/release/orbit-axion.exe dist/OrbitAxion-0.2.5-x64.msi 0.2.5
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

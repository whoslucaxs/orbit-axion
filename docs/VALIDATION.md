# Verificações locais — 0.2.5 — 2026-11-22

- 24 testes unitários: catálogo de ambas as bibliotecas, cores individuais, geometria central, compatibilidade JSON e verificações anteriores.
- Regressão de submenu em janela real: estado de botão mantido após transição e menu fechado na soltura sobre centro, vazio e alvo de submenu; modo clique preservado.
- UI test aplica cores de botão/ícone, seleciona ícone Bootstrap e salva o resultado, sem executar a ação.
- Capturas do renderizador inspecionadas para Bootstrap e Material, seletor visual, círculo central e efeito de vidro; testes com backend padrão e software.
- Teste de alpha do fundo detecta clamp incorreto de opacidade; cantos continuam transparentes e fundo nativo desativado.
- Fontes incorporadas verificadas no código gerado pelo Slint (registro por memória).

O seletor nativo ChooseColorW foi aberto em diagnóstico isolado, mostrou a paleta visual e aceitou RGB (255,172,172); a confirmação retornou #FFACAC, validando o fluxo real e a conversão de canais. Nenhuma configuração do usuário foi alterada por esse diagnóstico. O efeito Liquid Glass é visual, sem APIs Apple de refração. Instalação/upgrade/rollback do MSI em VM e testes prolongados em outros dispositivos continuam pendentes.

---

# Verificações locais — 0.2.4 — 2026-11-22

- 22 testes unitários: migração de cores, independência após reinício, validação RGB/RGBA e verificações anteriores.
- UI test seleciona F8 e F13 pela lista e verifica o código real escolhido.
- UI test salva a aparência do menu e exige que as demais preferências permaneçam idênticas.
- Comparação do buffer completo do menu antes/depois de alterar fundo, opacidade, destaque e tema do aplicativo.
- Snapshot da página Cores do menu em target/menu-colors-test.png; cantos do anel continuam transparentes.

A lista de teclas funciona sem armar os hooks. A validação não substitui testes de teclas proprietárias/reservadas em todo hardware.

---

# Verificações locais — 0.2.3 — 2026-11-22

- 18 testes unitários incluem tecla configurada, compatibilidade do JSON, persistência, pares de entrada, repetição e soltura após fechamento.
- UI test carrega F13 como acionamento e percorre os cinco botões do mouse mais teclas em modo clique/segurar, em janela real sem interceptação.
- Probe instala/remove hooks desarmados para mouse, F13 e F24.
- Entradas sintetizadas pelo executor recebem marcador para evitar reativação pelo próprio Orbit.
- Mantidos os testes de fundo nativo desativado, cantos transparentes, foco e geometria da 0.2.2.

A captura usa uma tecla individual, sem combinação. No editor a tecla de ativação fica livre para digitação. Atalhos reservados pelo sistema e variações de hardware/driver não foram validados em todos os ambientes.

---

# Verificações locais — 0.2.2 — 2026-11-22

- Desativado o backdrop DWM que preenchia os limites retangulares da janela do anel.
- Smoke consulta DWMWA_SYSTEMBACKDROP_TYPE na janela real e exige DWMSBT_NONE quando o atributo está disponível.
- Snapshot do anel verifica alpha zero nos quatro cantos; cor e opacidade continuam aplicadas ao círculo.
- Removida a linha decorativa que extrapolava a borda circular.
- Permanecem as verificações de foco, geometria, cinco botões e persistência.

O teste consulta o estado nativo e o buffer do renderizador; a aparência final em outros drivers/compositores exige validação nesses ambientes.

---

# Verificações locais — 0.2.1 — 2026-11-22

Ambiente: Windows 11 x64, Rust MSVC 1.98.1, Slint 1.19.0.

- 16 testes unitários, Rustfmt e Clippy com warnings como erros.
- Regressão reproduzida no controlador: um clique curto processado com atraso podia fechar o anel. O teste agora usa os instantes reais de pressionar/soltar, percorre cinco botões em modo clique/segurar e mantém a janela aberta por múltiplos eventos antes de verificar visibilidade/foco.
- A configuração de janela não podia usar o título para distinguir o anel: o hook do Slint recebe o nome genérico do processo nessa fase. A identificação passou para o momento de construção do componente.
- Removido o monitor assíncrono de foreground; o controlador usa o foreground atual a cada frame, enquanto o hook deixa de interceptar imediatamente quando a origem não é mais foreground.
- Teste de interface verifica cor/opacidade carregadas; snapshot do renderizador confirmou RGB (70,69,69) e alpha 94 para #464545 com 37% de opacidade. Imagens de teste ficam somente em target.
- JSON legado sem background_color continua válido. Teste de armazenamento verifica persistência da cor e opacidade.
- Cinco códigos de botão são testados com seus pares pressionar/soltar. Botões específicos do fabricante além dos cinco padrões dependem de mapeamento pelo driver.

A validação do controlador não injeta mouse e não instala hooks ativos. Testes físicos prolongados com o mouse do usuário e instalação/rollback do MSI em VM continuam necessários. O roteiro geral está em TEST_PLAN.md.

---

# Verificações locais — 0.2.0 — 2026-11-22

Ambiente: Windows 11 x64, Rust MSVC 1.98.1, Slint 1.19.0, Python 3.9.

- Rustfmt e Clippy com warnings como erros: aprovados.
- 14 testes unitários: aprovados (core, plataforma, áudio, executor, updater e armazenamento).
- UI test: valores de preferências preservados ao criar a página.
- Smoke real: anel visível, NOACTIVATE, TOPMOST, centro exato de 360 px e foreground preservado; aprovado com backend padrão e com SLINT_BACKEND=winit-software. O cenário inicial exibia também o editor e teve interferência de sua ativação; corrigido para testar somente o overlay.
- Probe: três botões instalaram/removeram hooks sem interceptar; leitura Core Audio e Potato Strip 0/7, Bus 0/4 bem-sucedida, sem alterar valores.
- Build otimizado Windows x64: aprovado.
- Empacotador MSI extrai o CAB e exige hashes idênticos aos arquivos de origem.

A inspeção visual final foi interrompida pelo usuário com Esc. A interface já havia sido inspecionada durante o desenvolvimento; a última compilação não recebeu nova inspeção visual completa. Não foram executados comandos de desligamento/reinicialização nem instalação/upgrade/rollback do MSI no computador pessoal. Essas verificações continuam no roteiro manual, junto a DPI misto, jogos, elevação e acessibilidade.

# Changelog

## 0.2.8 — ícone automático do programa — 2026-11-23

- Selecionar ou colar o caminho de um EXE/LNK na ação aplica automaticamente seu ícone ao botão, mantendo os filtros ajustáveis.
- Extração em segundo plano com salvamento habilitado após conclusão e descarte de resultados de destinos anteriores.

## 0.2.7 — ícones de executáveis e filtros — 2026-11-23

- Botão próprio para extrair ícones de executáveis e atalhos; abre os ajustes com preto e branco em 100%.
- Prévia com intensidade de escala de cinza, brilho, contraste e inversão de cores, salvos por botão.
- Filtros reversíveis, sem modificar a imagem de origem; também disponíveis para imagens importadas.
- Carregamento assíncrono preserva a prévia dos ajustes ainda não salvos.

## 0.2.6 — ícones de aplicativos — 2026-11-23

- SVGs do Voicemeeter e ChatGPT incorporados ao executável e à biblioteca de ícones.
- Categoria Aplicativos, busca por nome e seleção para os botões do menu.

## 0.2.5 — submenus e personalização visual — 2026-11-22

- Submenus por permanência em modo segurar; soltura fecha a navegação, inclusive sobre centro, vazio e outro submenu.
- Seletor nativo de cores com controles de opacidade; campos HEX manuais substituídos na interface.
- Fundo, borda e ícone configuráveis individualmente por botão.
- Círculo central com cor, borda e raio; zona central considera o raio configurado.
- Opções independentes de Liquid Glass visual para app e menu, com intensidade/reflexos no menu.
- Biblioteca offline de 4.312 ícones Bootstrap/Material, fontes incorporadas, busca e paginação.
- Correção da normalização da transparência dos canais de cor no Slint.

## 0.2.4 — paleta independente e seleção de teclas — 2026-11-22

- Paleta própria do menu para fundo, botões, ícones, textos, bordas, seleção e sombra; opacidade independente.
- Página Cores do menu com prévia e salvamento separado das preferências do aplicativo.
- Migração única preserva as cores antigas; alterações posteriores no app não afetam o menu.
- Lista de teclas no editor com seleção inicial F13, sem depender da captura global.
- Testes comparam o render do menu antes/depois de mudar a paleta do app e verificam seleção F8/F13 pela interface.

## 0.2.3 — acionamento pelo teclado — 2026-11-22

- Captura de teclas do teclado, incluindo letras, funções F1–F24 e modificadores, além dos botões do mouse.
- Escolha Mouse/Teclado com nome da tecla, persistência e compatibilidade com configurações anteriores.
- Repetições e soltura pertencem à tecla capturada; digitação permanece livre no editor.
- Entrada sintetizada pelas próprias ações é identificada para não reativar o menu.
- Mantida a correção do fundo externo transparente da versão 0.2.2.

## 0.2.2 — anel sem fundo retangular — 2026-11-22

- Desativado o backdrop acrílico nativo que preenchia toda a janela retangular do anel.
- Removida a linha decorativa que ultrapassava a borda circular.
- Mantida a cor/opacidade no círculo; teste verifica backdrop nativo desativado e cantos transparentes.

## 0.2.1 — abertura e aparência — 2026-11-22

- Overlay identificado durante a construção, antes de o Slint atribuir seu título; aplica abertura sem ativação corretamente.
- Tempo de clique medido pelos instantes dos eventos, independente de atrasos da interface.
- Removida a fila assíncrona de eventos antigos de foreground; estado atual é verificado a cada frame do anel.
- Ativação por cinco botões padrão, incluindo esquerdo/direito; cliques de configuração permanecem livres no editor.
- Cor HEX do fundo e opacidade de 0 a 100%, com prévia e persistência compatível com configurações anteriores.
- Teste do controlador em janela real para evitar regressão do anel que pisca.

## 0.2.0 — aplicativo e editor — 2026-11-22

- Editor de ações, perfis automáticos/manuais, submenus e macros.
- Persistência atômica, backups e pacotes portáteis com ícones.
- Bandeja, instância única, inicialização opcional, watchdog e modo seguro.
- Endpoints Windows e todos os strips/buses Potato, com feedback de disponibilidade.
- Tema Inked claro/escuro com vidro discreto e destaque turquesa.
- Updater GitHub com verificação de digest e consentimento; empacotamento MSI.

## 0.1.0 — laboratório técnico — 2026-11-22

- Workspace Rust com núcleo, adaptador Windows, áudio e aplicativo Slint.
- Overlay de oito posições e captura opcional de MButton/XBUTTON1/XBUTTON2.
- Protótipos de clique, hold/soltar, wheel e texto ASCII sem ativação do overlay.
- Worker de áudio Core Audio/Voicemeeter Potato e diagnósticos somente leitura.
- Testes de geometria, normalização de wheel, supressão de soltura e limites de áudio.
- Relatório de viabilidade e pendências antes do MVP.

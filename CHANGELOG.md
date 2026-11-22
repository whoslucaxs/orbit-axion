# Changelog

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

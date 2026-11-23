# Dependências e distribuição

O projeto usa a opção GPL-3.0-only do Slint 1.19.0 e publica seu código sob GPL-3.0-only. windows-rs, serde, serde_json, image, rfd, ureq, rustls, sha2, semver, base64, unicode-normalization, libloading e dependências transitivas mantêm suas licenças próprias.

`python packaging/windows-msi/collect_notices.py` gera `dist/THIRD_PARTY_NOTICES.txt` a partir das versões resolvidas no Cargo.lock, declarações dos manifests e arquivos LICENSE/COPYING/NOTICE presentes no cache Cargo. O MSI incorpora esse arquivo e a GPL. O inventário inclui dependências de build e plataformas além do Windows; não é uma auditoria jurídica nem prova de que todo aviso transitivo foi identificado. Revise-o antes de distribuição pública.

A DLL Voicemeeter Remote API é carregada exclusivamente da instalação local registrada e não é incluída no código, MSI ou repositório. Revise os termos VB-Audio para qualquer distribuição do SDK. Nenhum recurso da Logitech ou fonte proprietária é copiado. Ícones importados pelo usuário continuam sob responsabilidade de seu titular.

O repositório fornece Cargo.lock e fontes correspondentes. Ao redistribuir binários, forneça também o código correspondente e os avisos exigidos pelas licenças aplicáveis.

## Bibliotecas de ícones incorporadas

Bootstrap Icons (MIT) e Material Icons (Apache-2.0) são distribuídos como fontes e catálogo em assets/icons. As revisões exatas, contagens e conversão WOFF2→TTF estão em assets/icons/SOURCES.json. As licenças completas acompanham os arquivos e são agregadas ao NOTICES.txt do MSI. As fontes são incorporadas ao executável pelo compilador Slint, sem instalação de fontes no Windows.

A referência liquid-glass-skill é voltada a SwiftUI/Apple. O tema deste projeto é uma adaptação visual independente em Slint, sem código SwiftUI nem refração nativa Apple.

## Instalador

WiX Toolset 5.0.2 e sua extensão de UI (MS-RL) são ferramentas de build fixadas por versão e SHA-256. A licença está em `packaging/windows-msi/WIX-LICENSE.txt` e acompanha o inventário distribuído no MSI. O instalador usa os diálogos originais da biblioteca WixUI, com a navegação de escopo/pasta definida em `Package.wxs`; não é um bundle Burn.

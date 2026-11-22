# Dependências e distribuição

O projeto usa a opção GPL-3.0-only do Slint 1.19.0 e publica seu código sob GPL-3.0-only. windows-rs, serde, serde_json, image, rfd, ureq, rustls, sha2, semver, base64, unicode-normalization, libloading e dependências transitivas mantêm suas licenças próprias.

`python packaging/windows-msi/collect_notices.py` gera `dist/THIRD_PARTY_NOTICES.txt` a partir das versões resolvidas no Cargo.lock, declarações dos manifests e arquivos LICENSE/COPYING/NOTICE presentes no cache Cargo. O MSI incorpora esse arquivo e a GPL. O inventário inclui dependências de build e plataformas além do Windows; não é uma auditoria jurídica nem prova de que todo aviso transitivo foi identificado. Revise-o antes de distribuição pública.

A DLL Voicemeeter Remote API é carregada exclusivamente da instalação local registrada e não é incluída no código, MSI ou repositório. Revise os termos VB-Audio para qualquer distribuição do SDK. Nenhum recurso da Logitech ou fonte proprietária é copiado. Ícones importados pelo usuário continuam sob responsabilidade de seu titular.

O repositório fornece Cargo.lock e fontes correspondentes. Ao redistribuir binários, forneça também o código correspondente e os avisos exigidos pelas licenças aplicáveis.

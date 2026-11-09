# AnyCue: aplicativo para Windows

**Este projeto não é um website.** O AnyCue é um aplicativo desktop Tauri 2 (Rust + Svelte 5), com interface WebView2 **empacotada dentro do EXE**. Ele não precisa de servidor HTTP/IP para funcionar. A porta `127.0.0.1:1420` só é usada em desenvolvimento (`tauri dev`).

## Download do EXE

Abra a [release v0.1.0 Beta](https://github.com/AthilioRecovery/AnyCue/releases/tag/v0.1.0) e baixe:

- `AnyCue.exe`: portátil (requer WebView2, normalmente disponível no Windows 11).
- `AnyCue_0.1.0_x64-setup.exe`: instalador Windows NSIS.
- `SHA256SUMS.txt`: checksums SHA-256 para verificar os arquivos.

A compilação acontece pelo workflow Windows do GitHub Actions. Caso a release não esteja disponível, consulte a aba **Actions** para identificar falhas de build.

## Compilar localmente

Instale Rust (MSVC), Visual Studio Build Tools com Desktop development with C++, WebView2 e Node.js 22; em seguida execute `COMPILAR-WINDOWS.bat`.

## Funcionalidades atuais

- Janela desktop nativa (sem navegador externo ou porta aberta em produção).
- Biblioteca local persistente, coleções, importação, busca, favoritos e edição.
- Reprodução via dispositivos Windows escolhidos, saída virtual opcional e mixagem experimental de mic real para saída virtual.
- Volume por som, volume geral, VU aproximado, atalhos 1–9 **quando a janela está focada**, Esc para parar.

## Limitações conhecidas

- Saída para microfone do Discord/jogos depende de um driver virtual como VB-CABLE configurado no Windows.
- Hotkeys globais, Auto Keys/PTT, normalização LUFS, gravação, TTS e edição de onda **ainda não estão implementados**.
- A entrada de microfone é mono; não há conversão explícita de taxa de amostragem entre mic e saída virtual.
- A engine atual não é otimizada para grandes arquivos e não garante latência mínima.
- Recompilar Windows em GitHub Actions é a fonte da validação do executável; criar o código em Linux não prova que Windows já passou no build.

## Frontend

O layout foi adaptado do visual do `frontend.7z` fornecido pelo autor: paleta cinza-escuro `#252525`, botões compactos, controles superiores, formulário de configurações em duas colunas e visualização de estado. Não foram copiadas funções de legendas, traduções, modelos ou seus arquivos de idioma, que são irrelevantes para um soundboard.

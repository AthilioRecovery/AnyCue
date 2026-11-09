<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { open } from '@tauri-apps/plugin-dialog';

  type Destination = 'monitor' | 'virtual' | 'both';
  type Sound = { id: string; title: string; path: string; category: string; color: string; favorite: boolean; volume: number; destination: Destination };
  type Library = { version: number; categories: string[]; sounds: Sound[] };
  type Device = { name: string; direction: 'input' | 'output'; isDefault: boolean };
  type Tab = 'sounds' | 'routing' | 'settings';

  let tab: Tab = 'sounds';
  let library: Library = { version: 1, categories: ['Geral'], sounds: [] };
  let devices: Device[] = [];
  let category = 'all';
  let search = '';
  let onlyFavorites = false;
  let microphone = '';
  let monitor = '';
  let virtualOutput = '';
  let volume = 85;
  let current: Sound | null = null;
  let playing = false;
  let duration = 0;
  let elapsed = 0;
  let playedAt = 0;
  let peak = 0;
  let notice = '';
  let connected = false;
  let loading = true;
  let editSound: Sound | null = null;
  let editTitle = '';
  let editCategory = '';
  let editVolume = 100;
  let editDestination: Destination = 'both';
  let editColor = '#b8b8b8';
  let mounted = true;

  $: inputs = devices.filter(d => d.direction === 'input');
  $: outputs = devices.filter(d => d.direction === 'output');
  $: rows = library.sounds.filter(s =>
    (category === 'all' || s.category === category) &&
    (!onlyFavorites || s.favorite) &&
    `${s.title} ${s.category}`.toLocaleLowerCase().includes(search.toLocaleLowerCase())
  );
  const bars = [30, 55, 40, 80, 68, 26, 53, 90, 48, 64, 36, 78, 54, 92, 42, 72, 38, 65, 86, 50];
  const fmt = (v: number) => `${Math.floor(v / 60)}:${String(Math.floor(v % 60)).padStart(2, '0')}`;
  const report = (e: unknown) => { notice = String(e); };
  async function reload() { library = await invoke<Library>('get_library'); }
  async function refreshDevices() {
    devices = await invoke<Device[]>('list_devices');
    if (!monitor) monitor = outputs.find(d => d.isDefault)?.name || outputs[0]?.name || '';
    if (!microphone) microphone = inputs.find(d => d.isDefault)?.name || '';
  }
  async function applyRouting() {
    try {
      if (!monitor && !virtualOutput) throw new Error('Escolha pelo menos uma saída de áudio.');
      await invoke('configure_audio', { microphone: microphone || null, monitor: monitor || null, virtualOutput: virtualOutput || null });
      await invoke('set_master_volume', { volume });
      connected = true;
      notice = 'Dispositivos de áudio conectados.';
    } catch (e) { connected = false; report(e); }
  }
  async function importSounds() {
    try {
      const files = await open({ multiple: true, filters: [{ name: 'Áudio', extensions: ['mp3', 'wav', 'ogg', 'flac', 'opus', 'aac', 'm4a'] }] });
      const paths = Array.isArray(files) ? files : files ? [files] : [];
      if (!paths.length) return;
      await invoke('add_files', { paths, category: category === 'all' ? 'Geral' : category });
      await reload();
      tab = 'sounds';
      notice = `${paths.length} som(ns) importado(s).`;
    } catch (e) { report(e); }
  }
  async function play(sound: Sound) {
    try {
      duration = await invoke<number>('play_sound', { id: sound.id });
      current = sound; playing = true; elapsed = 0; playedAt = performance.now();
      notice = '';
    } catch (e) { report(e); }
  }
  async function stop() {
    await invoke('stop_all').catch(report);
    playing = false; elapsed = 0;
  }
  async function setVolume(value: number) {
    volume = value;
    await invoke('set_master_volume', { volume }).catch(report);
  }
  async function toggleFavorite(sound: Sound) {
    try { await invoke('update_sound', { sound: { ...sound, favorite: !sound.favorite } }); await reload(); }
    catch (e) { report(e); }
  }
  function edit(sound: Sound) {
    editSound = sound;
    editTitle = sound.title;
    editCategory = sound.category;
    editVolume = sound.volume;
    editDestination = sound.destination;
    editColor = sound.color;
  }
  async function saveEdit() {
    if (!editSound) return;
    try {
      await invoke('update_sound', { sound: { ...editSound, title: editTitle.trim() || editSound.title, category: editCategory, volume: editVolume, destination: editDestination, color: editColor } });
      editSound = null; await reload();
    } catch (e) { report(e); }
  }
  async function deleteSound() {
    if (!editSound || !confirm('Remover este som da biblioteca? O arquivo original não será apagado.')) return;
    try {
      const id = editSound.id;
      await invoke('delete_sound', { id });
      if (current?.id === id) { await stop(); current = null; }
      editSound = null; await reload();
    } catch (e) { report(e); }
  }
  async function newCategory() {
    const name = prompt('Nome da coleção:')?.trim();
    if (!name) return;
    try { await invoke('add_category', { name }); await reload(); category = name; }
    catch (e) { report(e); }
  }
  function keyDown(e: KeyboardEvent) {
    if ((e.target as HTMLElement)?.closest('input,textarea,select') || e.ctrlKey || e.altKey || e.metaKey) return;
    if (e.key === 'Escape') { if (editSound) editSound = null; else void stop(); return; }
    if (tab === 'sounds' && /^[1-9]$/.test(e.key)) { const sound = rows[Number(e.key) - 1]; if (sound) { e.preventDefault(); void play(sound); } }
  }
  function windowAction(action: 'minimize' | 'close') { void getCurrentWindow()[action]().catch(report); }
  onMount(() => {
    mounted = true;
    const init = async () => {
      try { await reload(); await refreshDevices(); if (monitor) await applyRouting(); }
      catch (e) { report(e); }
      finally { loading = false; }
    };
    void init();
    window.addEventListener('keydown', keyDown);
    const timer = setInterval(() => {
      if (playing) { elapsed = Math.min(duration, (performance.now() - playedAt) / 1000); if (elapsed >= duration) playing = false; }
      if (mounted) void invoke<number>('get_peak').then(v => peak = v).catch(() => {});
    }, 160);
    return () => { mounted = false; clearInterval(timer); window.removeEventListener('keydown', keyDown); };
  });
</script>

<svelte:head><title>AnyCue | Windows Soundboard</title></svelte:head>
<div class="overlay">
  <header class="toolbar">
    <div class="brand" data-tauri-drag-region><span class="brand-mark" data-tauri-drag-region>ac</span><span class="brand-name" data-tauri-drag-region>AnyCue</span></div>
    <div class="drag-region" data-tauri-drag-region></div>
    <div class="controls">
      <button class="btn primary" on:click={importSounds} title="Importar áudio">＋ <span>Importar</span></button>
      <button class="btn" on:click={stop} title="Parar todos os sons">■ <span>Parar</span></button>
      <button class="icon-btn" aria-label="Minimizar" title="Minimizar" on:click={() => windowAction('minimize')}>−</button>
      <button class="icon-btn close-button" aria-label="Fechar" title="Fechar AnyCue" on:click={() => windowAction('close')}>×</button>
    </div>
  </header>
  <div class="tabs" role="tablist" aria-label="Seções do aplicativo">
    <button class:active={tab === 'sounds'} on:click={() => tab = 'sounds'}>Biblioteca</button>
    <button class:active={tab === 'routing'} on:click={() => tab = 'routing'}>Roteamento</button>
    <button class:active={tab === 'settings'} on:click={() => tab = 'settings'}>Configurações</button>
    <span class="tab-status"><i class:connected></i>{connected ? 'Áudio conectado' : 'Verificar áudio'}</span>
  </div>
  <main class="content">
    {#if tab === 'sounds'}
      <div class="library-view">
        <div class="section-heading"><div><h1>Biblioteca de sons <span class="count">{library.sounds.length}</span></h1><p>Seu soundboard local. Escolha um som para reproduzir.</p></div><button class="btn" on:click={newCategory}>＋ Coleção</button></div>
        <div class="search-row"><input class="input search" type="search" placeholder="Buscar sons..." bind:value={search} aria-label="Buscar sons"/><select class="select" aria-label="Filtrar categoria" bind:value={category}><option value="all">Todas as coleções</option>{#each library.categories as c}<option value={c}>{c}</option>{/each}</select><label class="check"><input type="checkbox" bind:checked={onlyFavorites}/> Favoritos</label></div>
        {#if loading}<div class="empty">Carregando biblioteca...</div>{:else if rows.length === 0}
          <div class="empty"><div class="empty-symbol">♫</div><h2>Nenhum som encontrado</h2><p>Importe WAV, MP3, OGG, FLAC, AAC ou M4A para começar.</p><button class="btn primary" on:click={importSounds}>＋ Importar sons</button></div>
        {:else}
          <div class="sound-list">
            {#each rows as sound, i (sound.id)}
              <article class:playing={current?.id === sound.id && playing} class="sound-row">
                <button class="play-square" title={`Tocar ${sound.title}`} aria-label={`Tocar ${sound.title}`} on:click={() => void play(sound)}>{current?.id === sound.id && playing ? '▮▮' : '▶'}</button>
                <div class="sound-details"><strong title={sound.title}>{sound.title}</strong><small>{sound.category} · {sound.destination === 'both' ? 'Monitor + virtual' : sound.destination === 'monitor' ? 'Monitor' : 'Virtual'}</small></div>
                <div class="waveform" aria-hidden="true">{#each bars as height}<span style:height={`${height}%`}></span>{/each}</div>
                {#if i < 9}<kbd>{i + 1}</kbd>{/if}
                <button class:favorite={sound.favorite} class="icon-btn" title="Favorito" aria-label="Favoritar" on:click={() => toggleFavorite(sound)}>{sound.favorite ? '★' : '☆'}</button>
                <button class="icon-btn" title="Editar" aria-label={`Editar ${sound.title}`} on:click={() => edit(sound)}>⋯</button>
              </article>
            {/each}
          </div>
        {/if}
      </div>
    {:else if tab === 'routing'}
      <div class="settings"><h2>Roteamento de áudio</h2><p class="hint">Configure o dispositivo do Windows usado como alto-falante e, opcionalmente, uma saída virtual (VB-CABLE).</p>
        <div class="setting-row"><label for="mic">Microfone real</label><select id="mic" class="select" bind:value={microphone}><option value="">Não capturar</option>{#each inputs as d}<option value={d.name}>{d.name}{d.isDefault ? ' (padrão)' : ''}</option>{/each}</select></div>
        <div class="setting-row"><label for="monitor">Alto-falantes</label><select id="monitor" class="select" bind:value={monitor}><option value="">Desativado</option>{#each outputs as d}<option value={d.name}>{d.name}{d.isDefault ? ' (padrão)' : ''}</option>{/each}</select></div>
        <div class="setting-row"><label for="virtual">Saída virtual</label><select id="virtual" class="select" bind:value={virtualOutput}><option value="">Desativada</option>{#each outputs as d}<option value={d.name}>{d.name}</option>{/each}</select></div>
        <div class="setting-row"><span>Nível de saída</span><div class="meter"><span style:width={`${Math.min(100, peak * 100)}%`}></span></div></div>
        <div class="settings-footer"><span class="settings-credit">O AnyCue não instala drivers e não abre portas de rede.</span><button class="btn" on:click={() => void refreshDevices().catch(report)}>Atualizar</button><button class="btn primary" on:click={applyRouting}>Aplicar</button></div>
      </div>
    {:else}
      <div class="settings"><h2>Configurações</h2><h3 class="settings-subtitle">Reprodução</h3>
        <div class="setting-row"><label for="volume">Volume principal</label><div class="range-control"><input id="volume" type="range" min="0" max="100" value={volume} on:input={e => void setVolume(Number(e.currentTarget.value))}/><output>{volume}%</output></div></div>
        <div class="setting-row"><span>Atalhos</span><span class="value">Teclas 1–9 (janela focada). Esc para parar.</span></div>
        <h3 class="settings-subtitle">Privacidade</h3><div class="setting-row"><span>Dados locais</span><span class="value">Biblioteca salva neste computador. Sem telemetria.</span></div>
        <div class="setting-row"><span>Microfone virtual</span><span class="value">É necessário instalar e selecionar um cabo virtual externo.</span></div>
        <div class="setting-row"><span>Versão</span><span class="value">AnyCue 0.1.1 alpha · MIT · Windows 10/11</span></div>
        <div class="settings-footer"><span class="settings-credit">Interface compacta inspirada no frontend fornecido.</span><button class="btn primary" on:click={() => tab = 'routing'}>Configurar áudio</button></div>
      </div>
    {/if}
  </main>
  <footer class="statusbar"><div class="now-playing"><span class="status-symbol">{playing ? '▶' : '■'}</span><div><b>{current?.title || 'Nenhum som em reprodução'}</b><small>{playing ? 'Reproduzindo' : 'Pronto'} · {fmt(elapsed)} / {fmt(duration)}</small></div></div><div class="status-actions"><span>Volume</span><input aria-label="Volume" type="range" min="0" max="100" value={volume} on:input={e => void setVolume(Number(e.currentTarget.value))}/><output>{volume}%</output><button class="btn" on:click={stop}>■ Parar</button></div></footer>
  {#if notice}<div class="notice" role="status"><span>{notice}</span><button aria-label="Fechar aviso" on:click={() => notice = ''}>×</button></div>{/if}
  {#if editSound}
    <div class="modal-backdrop" role="presentation" on:click={e => { if (e.target === e.currentTarget) editSound = null; }}>
      <div class="modal" role="dialog" aria-modal="true" aria-label="Editar som"><h2>Editar som</h2>
        <div class="setting-row"><label for="edit-name">Nome</label><input class="input" id="edit-name" maxlength="80" bind:value={editTitle}/></div>
        <div class="setting-row"><label for="edit-category">Coleção</label><select id="edit-category" class="select" bind:value={editCategory}>{#each library.categories as c}<option value={c}>{c}</option>{/each}</select></div>
        <div class="setting-row"><label for="edit-output">Destino</label><select id="edit-output" class="select" bind:value={editDestination}><option value="both">Ambas as saídas</option><option value="monitor">Apenas monitor</option><option value="virtual">Apenas virtual</option></select></div>
        <div class="setting-row"><label for="edit-volume">Volume</label><div class="range-control"><input id="edit-volume" type="range" min="0" max="200" bind:value={editVolume}/><output>{editVolume}%</output></div></div>
        <div class="setting-row"><label for="edit-color">Cor</label><input type="color" id="edit-color" bind:value={editColor}/></div>
        <div class="settings-footer"><button class="btn danger" on:click={deleteSound}>Excluir</button><div class="spacer"></div><button class="btn" on:click={() => editSound = null}>Cancelar</button><button class="btn primary" on:click={saveEdit}>Salvar</button></div>
      </div>
    </div>
  {/if}
</div>

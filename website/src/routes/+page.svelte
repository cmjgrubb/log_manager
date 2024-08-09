<script lang="ts">
	import { onMount } from 'svelte';

	interface LogEntry {
		timestamp: string;
		hostname: string;
		log_level: string;
		message: string;
	}

	let logs: LogEntry[] = [];
	let hostname = '';
	let log_level = '';
	let message = '';

	onMount(async () => {
		const response = await fetch(
			`/logs?hostname=${hostname}&log_level=${log_level}&message=${message}`
		);
		logs = await response.json();
	});

	function updateLogs() {
		onMount(async () => {
			const response = await fetch(
				`/logs?hostname=${hostname}&log_level=${log_level}&message=${message}`
			);
			logs = await response.json();
		});
	}
</script>

<div>
	<label>Hostname: <input type="text" bind:value={hostname} /></label>
	<label>Log Level: <input type="text" bind:value={log_level} /></label>
	<label>Message: <input type="text" bind:value={message} /></label>
	<button on:click={updateLogs}>Search</button>
</div>
<ul>
	{#each logs as log}
		<li>
			<strong>{log.timestamp}</strong> - {log.hostname} - {log.log_level} - {log.message}
		</li>
	{/each}
</ul>

<style>
	:global(body) {
		font-family: Arial, sans-serif;
		font-size: 16px;
		margin: 1.5rem;
		padding: 0rem;
		line-height: 2;
	}

	button {
		background-color: #014b92;
		color: #fff;
		padding: 0.5rem 1rem;
		border: none;
		margin-top: 2rem;
	}

	h2 {
		color: #3a6eab;
		font-size: 1.5rem;
		margin-top: 2rem;
		margin-bottom: 0.5rem;
	}

	h3 {
		color: #3a6eab;
		font-size: 1.3rem;
		margin-top: 2rem;
		margin-bottom: 0.5rem;
	}
</style>

<script lang="ts">
	import { activeVault } from '@/stores';
	import EncryptedVault from './EncryptedVault.svelte';
	import FileList from './FileList.svelte';
</script>

{#if $activeVault && $activeVault.is_encrypted}
	<EncryptedVault {...$activeVault}></EncryptedVault>
{:else if $activeVault && !$activeVault.is_encrypted}
	<FileList vault={$activeVault}></FileList>
{/if}

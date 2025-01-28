<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index';
	import * as Collapsible from '$lib/components/ui/collapsible/index';
	import { ChevronUp, Trash2, LogOut, Box, FolderLock } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { fetchVaults, type Vault } from '@/api/vaults';
	import { activeVault, vaultStore } from '@/stores';
	import NewVaultDialog from './NewVaultDialog.svelte';

	let vaultsExpanded = $state(true);

	onMount(async () => {
		const vaults = await fetchVaults();
		vaultStore.set(vaults);
	});

	function selectVault(v: Vault) {
		activeVault.set(v);
	}
</script>

<Sidebar.Root>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Linkyhost</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					<Collapsible.Root bind:open={vaultsExpanded} class="group/collapsible">
						<Sidebar.MenuItem>
							<Collapsible.Trigger>
								{#snippet child({ props })}
									<Sidebar.MenuButton {...props} class="items-center">
										<Box></Box>
										<span>Vaults</span>

										<NewVaultDialog></NewVaultDialog>
									</Sidebar.MenuButton>
								{/snippet}
							</Collapsible.Trigger>
							<Collapsible.Content>
								<Sidebar.MenuSub>
									{#each $vaultStore as vault}
										<Sidebar.MenuSubItem
											onclick={() => selectVault(vault)}
											class="hover:bg-secondary flex select-none flex-row items-center justify-between break-words rounded-md px-2 py-1 transition-colors"
										>
											<span>{vault.name}</span>

											{#if vault.is_encrypted}
												<FolderLock size="14"></FolderLock>
											{/if}
										</Sidebar.MenuSubItem>
									{/each}
								</Sidebar.MenuSub>
							</Collapsible.Content>
						</Sidebar.MenuItem>
					</Collapsible.Root>
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>

	<Sidebar.Footer>
		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
					<Sidebar.MenuButton
						{...props}
						class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
					>
						Linker
						<ChevronUp class="ml-auto" />
					</Sidebar.MenuButton>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content side="top" class="w-[--bits-dropdown-menu-anchor-width]">
				<DropdownMenu.Item>
					<Trash2></Trash2>
					<span>Delete Account</span>
				</DropdownMenu.Item>
				<DropdownMenu.Item>
					<LogOut></LogOut>
					<span>Sign out</span>
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</Sidebar.Footer>
</Sidebar.Root>

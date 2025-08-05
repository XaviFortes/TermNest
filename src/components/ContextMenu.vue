<template>
  <div 
    class="context-menu" 
    v-if="show"
    :style="{ top: position.y + 'px', left: position.x + 'px' }"
    @click.stop
  >
    <div class="context-menu-item" @click="$emit('connect')">
      <span class="menu-icon">🔗</span>
      Connect
    </div>
    <div class="context-menu-item" @click="$emit('new-connection')">
      <span class="menu-icon">🚀</span>
      New Connection
    </div>
    <div class="context-menu-item" @click="$emit('edit')">
      <span class="menu-icon">✏️</span>
      Edit
    </div>
    <div class="context-menu-item" @click="$emit('duplicate')">
      <span class="menu-icon">📋</span>
      Duplicate
    </div>
    <div class="context-menu-separator"></div>
    <div class="context-menu-item" @click="$emit('export')">
      <span class="menu-icon">📤</span>
      Export
    </div>
    <!-- <div class="context-menu-separator"></div>
    <div class="context-menu-item" @click="$emit('test-connected')">
      <span class="menu-icon">🟢</span>
      Test Connected
    </div>
    <div class="context-menu-item" @click="$emit('test-connecting')">
      <span class="menu-icon">🟡</span>
      Test Connecting
    </div>
    <div class="context-menu-item" @click="$emit('test-error')">
      <span class="menu-icon">🔴</span>
      Test Error
    </div> -->
    <div class="context-menu-separator"></div>
    <div class="context-menu-item danger" @click="$emit('delete')">
      <span class="menu-icon">🗑️</span>
      Delete
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  show: boolean
  position: { x: number; y: number }
}>()

defineEmits<{
  connect: []
  'new-connection': []
  edit: []
  duplicate: []
  export: []
  delete: []
  'test-connected': []
  'test-connecting': []
  'test-error': []
}>()
</script>

<style scoped>
.context-menu {
  position: fixed;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.25);
  z-index: 9999;
  padding: 0.5rem 0;
  min-width: 160px;
  max-width: 250px;
  backdrop-filter: blur(10px);
  overflow: hidden;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  cursor: pointer;
  transition: background-color 0.2s ease;
  color: var(--text-primary);
  font-size: 0.875rem;
}

.context-menu-item:hover {
  background: var(--bg-secondary);
}

.context-menu-item.danger {
  color: #dc3545;
}

.context-menu-item.danger:hover {
  background: rgba(220, 53, 69, 0.1);
}

.menu-icon {
  font-size: 1rem;
  width: 16px;
  text-align: center;
}

.context-menu-separator {
  height: 1px;
  background: var(--border-color);
  margin: 0.5rem 0;
}
</style>

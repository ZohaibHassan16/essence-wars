<script lang="ts">
  import "../app.css";

  let { children } = $props();
</script>

<div class="min-h-screen bg-ui-bg text-ui-text">
  {@render children()}
</div>

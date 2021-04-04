<template>
  <nav class="navbar navbar-dark">
    <button v-on:click="next">Next</button>
    <button v-on:click="toggle_open" v-if="is_open">Close</button>
    <button v-on:click="toggle_open" v-else>Open</button>
    <input
      class="form-control form-control-dark"
      v-model="pop_size"
      placeholder="Party size"
    />
  </nav>
</template>

<script>
export default {
    name: "QueueControls"
};
</script>

<style>
</style>
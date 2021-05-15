<template>
  <nav class="navbar navbar-dark">
    <button v-on:click="next">Next</button>
    <button v-on:click="toggle_open" v-if="is_open">Close</button>
    <button v-on:click="toggle_open" v-else>Open</button>
    <!-- <button v-on:click="add_entry">Add</button> -->
    <input
      class="form-control form-control-dark"
      v-model="pop_size"
      placeholder="Party size"
    />
  </nav>
  <!-- Button trigger modal -->
<button type="button" class="btn btn-primary" data-bs-toggle="modal" data-bs-target="#exampleModal">
  Launch demo modal
</button>

<!-- Modal -->
<div class="modal fade" id="exampleModal" tabindex="-1" aria-labelledby="exampleModalLabel" aria-hidden="true">
  <div class="modal-dialog">
    <div class="modal-content">
      <div class="modal-header">
        <h5 class="modal-title" id="exampleModalLabel">Modal title</h5>
        <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
      </div>
      <div class="modal-body">
        ...
      </div>
      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
        <button type="button" class="btn btn-primary">Save changes</button>
      </div>
    </div>
  </div>
</div>

</template>

<script>
export default {
    name: "QueueControls"
};
</script>

<style>
</style>
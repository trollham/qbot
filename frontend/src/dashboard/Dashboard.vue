<template>
  <div id="dashboard" class="container">
    <QueueControls />
    <Queue />
  </div>
</template>

<script>
import Queue from "../components/Queue.vue";
import QueueControls from "../components/QueueControls.vue";

export default {
  name: "Dashboard",
  components: {
    QueueControls,
    Queue,
  },
};
</script>

<style>
#dashboard {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>

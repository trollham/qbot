<template>
  <div id="dashboard" class="container">
    <QueueControls v-if="is_owner" />
    <Queue :is_owner="is_owner" :queue="queue" @remove-user="remove"/>
  </div>
</template>

<script>
import Queue from "../components/Queue.vue";
import QueueControls from "../components/QueueControls.vue";
const axios = require("axios").default;

export default {
  name: "Dashboard",
  components: {
    QueueControls,
    Queue,
  },
  data() {
    return {
      is_owner: false,
      queue: [],
    };
  },
  mounted() {
    const url = "/queue/" + this.$route.params.user;
    axios.get(url).then((response) => {
      console.log(response);
      this.is_owner = response.data.is_owner;
      this.queue = response.data.queue;
    });
  },
  methods: {
    remove(user) {
      console.log("Removing user: " + user);
      axios.delete("/queue/" + this.$route.params.user + "/entry/" + user.nickname)
      this.queue = this.queue.filter((u) => u !== user);
    },
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

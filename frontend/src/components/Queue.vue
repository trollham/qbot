<template>
  <div class="queue">
    <div>
      <nav class="navbar navbar-dark">
        <strong>Queue size</strong>{{ queue.length }}
        <strong>Time remaining</strong>
        {{ (Math.floor(queue.length / pop_size) + 1) * 5 }} minutes
      </nav>
    </div>
    <div>
      <table class="table table-sm table-hover table-striped">
        <thead>
          <tr>
            <th scope="col">#</th>
            <th scope="col">Name</th>
            <th scope="col">Time</th>
            <th scope="col">Actions</th>
          </tr>
        </thead>
        <tbody name="user-table">
          <transition-group>
            <QueueEntry
              v-for="(user, index) in queue"
              :key="user.id"
              :entry="user"
              :index="index + 1"
              class="queue-item"
            ></QueueEntry>
          </transition-group>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import QueueEntry from "./QueueEntry.vue";

let initial_queue = [];
if (process.env.NODE_ENV !== "production") {
  initial_queue = [
    {
      nickname: "Travis Willingham",
      id: "1",
      time_joined: "12:20:32",
    },
    {
      nickname: "Laura Bailey",
      id: "2",
      time_joined: "12:21:19",
    },
    {
      nickname: "Liam O'Brian",
      id: "3",
      time_joined: "1:31:22",
    },
    {
      nickname: "Sam Riegel",
      id: "4",
      time_joined: "1:31:01",
    },
    {
      nickname: "Ashley Johnson",
      id: "5",
      time_joined: "1:35:16",
    },
    {
      nickname: "Taliesin Jaffe",
      id: "6",
      time_joined: "3:46:51",
    },
    {
      nickname: "Marisha Ray",
      id: "7",
      time_joined: "8:56:12",
    },
  ];
}

export default {
  name: "Queue",
  components: { QueueEntry },
  data() {
    return {
      pop_size: 4,
      is_open: false,
      queue: initial_queue,
    };
  },
};
</script>

<style scoped>
.queue {
  text-align: center;
}
.user-table-enter-active,
.user-table-leave-active {
  transition: all 1s;
}
.user-table-enter, .user-table-leave-to /* .user-table-leave-active below version 2.1.8 */ {
  opacity: 0;
  transform: translateX(100px);
}
</style>

<template>
  <tr>
    <th scope="row">{{ index }}</th>
    <td>{{ entry.nickname }}</td>
    <td>{{ entry.time_joined }}</td>
    <td v-if="is_owner">
      <button
        @click="$emit('remove-user', user)"
        class="btn btn-outline-danger"
      >
        <font-awesome-icon icon="minus-circle" />
        Remove
      </button>
    </td>
  </tr>
</template>

<script>
export default {
  props: {
    entry: {
      required: true,
      type: Object,
    },
    index: {
      required: true,
      type: Number,
    },
    is_owner: {
      required: true,
      type: Boolean,
    },
  },
  emits: ["remove-user"],
};
</script>

<style scoped>
</style>
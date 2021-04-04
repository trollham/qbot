<template>
    <span>Welcome to the landing page for <strong>brittlq</strong>!</span>
    <a href='/dashboard'>
    <font-awesome-icon :icon='["fab", "twitch"]' />
      Click here
    </a> to go to the dashboard.
</template>

<script>
export default {
  name: "Home"
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>

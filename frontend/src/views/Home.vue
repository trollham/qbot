<template>
  <span>Welcome to the landing page for <strong>brittlq</strong>!</span>
  <a :href="auth_url">
    <font-awesome-icon :icon="['fab', 'twitch']" />
    Click here
  </a>
  to go to the dashboard.
</template>

<script>
let redirect_url = process.env.VUE_APP_API_URL + '/login-redirect'
let auth_url = 'https://id.twitch.tv/oauth2/authorize'
                + '?client_id=25hshmzbtpompde80gzfr9bkahb9sp'
                + '&response_type=code'
                + '&scope=openid%20user:read:email'
                + '&claims={"id_token":{"email": null,"email_verified":null,"preferred_username":null}}'
                + '&redirect_uri=' + redirect_url

export default {
  name: "Home",
  data() {
    return  { auth_url: auth_url};
  }
};
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

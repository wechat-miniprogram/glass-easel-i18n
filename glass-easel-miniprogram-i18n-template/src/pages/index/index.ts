Page({
  data: {
    lang: "en-us",
    a:1,
    b:2,
  },
  helloTap() {
    if(this.data.lang === "en-us"){
      this.setData({
        lang: "ja",
      })
    } else if(this.data.lang === "ja") {
      this.setData({
        lang: "zh-cn",
      })
    } else {
      this.setData({
        lang: "en-us",
      })
    }
  },
});

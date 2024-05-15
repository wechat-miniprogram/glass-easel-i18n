Page({
  data: {
    locale: 'en-us',
    a: 1,
    b: 2,
    status: 1,
    arr: ['item1', 'item2', 'item3'],
  },
  changeLocale() {
    if (this.data.locale === 'en-us') {
      this.setData({
        locale: 'ja',
      })
    } else if (this.data.locale === 'ja') {
      this.setData({
        locale: 'zh-cn',
      })
    } else {
      this.setData({
        locale: 'en-us',
      })
    }
  },
})

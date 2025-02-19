use glass_easel_i18n::search;

#[test]
fn basic() {
    const SRC: &'static str = r#"
        <!I18N>
        <block><view>一些文字</view></block>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "一些文字");
}

#[test]
fn missing() {
    const SRC: &'static str = r#"
        <!I18N>
        <view>全局的翻译</view>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "全局的翻译");
}

#[test]
fn sub_template() {
    const SRC: &'static str = r#"
        <!I18N>
        <template name="a">
            <view>一些文字</view>
        </template>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "一些文字");
}

#[test]
fn attributes() {
    const SRC: &'static str = r#"
        <!I18N>
        <view title="说明" other="其他"></view>
    "#;
    let res = search("TEST", SRC, &["title".into()]).unwrap();
    assert_eq!(res.output.join("|"), "说明");
}

#[test]
fn children() {
    const SRC: &'static str = r#"
        <!I18N>
        <div><!I18N translate-children>我<span style="color: red">爱</span>你</div>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "爱|我{{A}}你");
}

#[test]
fn placeholders() {
    const SRC: &'static str = r#"
        <!I18N>
        <view>{{ a }} 加 {{ b }} 得到 {{ a+b }}</view>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "{{A}} 加 {{B}} 得到 {{C}}");
}

#[test]
fn if_block() {
    const SRC: &'static str = r#"
        <!I18N>
        <view wx:if="{{item.status === 2}}">一些文字</view>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "一些文字");
}

#[test]
fn for_block() {
    const SRC: &'static str = r#"
        <!I18N>
        <view wx:for="{{ arr }}">一些文字</view>
    "#;
    let res = search("TEST", SRC, &[]).unwrap();
    assert_eq!(res.output.join("|"), "一些文字");
}

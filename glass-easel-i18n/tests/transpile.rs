use glass_easel_i18n::compile;

const TRANSLATE_FILE: &'static str = r#"

[en-us]
"一些文字" = "Some words"
"含属性的节点" = "Node with attributes"
"说明" = "explanation"
"我{{A}}你" = "I {{A}} You"
"爱" = "Love"
"{{A}} 加 {{B}} 得到 {{C}}" = "Add {{A}} to {{B}} to get {{C}}"

[ja]
"一些文字" = "いくつかのテキスト"
"含属性的节点" = "属性を持つノード"
"说明" = "説明する"
"我{{A}}你" = "愛してます"
"{{A}} 加 {{B}} 得到 {{C}}" = "{{A}} を {{B}} に追加すると、{{C}} が得られます"

["global.en-us"]
"全局的翻译" = "Global translation"

"#;

#[test]
fn basic() {
    const SRC: &'static str = r#"
        <!I18N>
        <block><view>一些文字</view></block>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><block><view>Some words</view></block></block><block wx:elif=\"{{locale===\"ja\"}}\"><block><view>いくつかのテキスト</view></block></block><block wx:else><block><view>一些文字</view></block></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn missing() {
    const SRC: &'static str = r#"
        <!I18N>
        <view>全局的翻译</view>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><view>Global translation</view></block><block wx:elif=\"{{locale===\"ja\"}}\"><view>全局的翻译</view></block><block wx:else><view>全局的翻译</view></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn sub_template() {
    const SRC: &'static str = r#"
        <!I18N>
        <template name="a">
            <view>一些文字</view>
        </template>
    "#;
    const OUT: &'static str = "<template name=\"a\"><block wx:if=\"{{locale===\"en-us\"}}\"><view>Some words</view></block><block wx:elif=\"{{locale===\"ja\"}}\"><view>いくつかのテキスト</view></block><block wx:else><view>一些文字</view></block></template><block wx:if=\"{{locale===\"en-us\"}}\"/><block wx:elif=\"{{locale===\"ja\"}}\"/><block wx:else/>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn attributes() {
    const SRC: &'static str = r#"
        <!I18N>
        <view title="说明" other="说明"></view>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><view title=\"explanation\" other=\"说明\"/></block><block wx:elif=\"{{locale===\"ja\"}}\"><view title=\"説明する\" other=\"说明\"/></block><block wx:else><view title=\"说明\" other=\"说明\"/></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &["title".into()]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn children() {
    const SRC: &'static str = r#"
        <!I18N>
        <div><!I18N translate-children>我<span style="color: red">爱</span>你</div>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><div>I <span style=\"color: red\">Love</span> You</div></block><block wx:elif=\"{{locale===\"ja\"}}\"><div>愛してます</div></block><block wx:else><div>我<span style=\"color: red\">爱</span>你</div></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn placeholders() {
    const SRC: &'static str = r#"
        <!I18N>
        <view>{{ a }} 加 {{ b }} 得到 {{ a+b }}</view>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><view>Add {{a}} to {{b}} to get {{a+b}}</view></block><block wx:elif=\"{{locale===\"ja\"}}\"><view>{{a}} を {{b}} に追加すると、{{a+b}} が得られます</view></block><block wx:else><view>{{a}} 加 {{b}} 得到 {{a+b}}</view></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn if_block() {
    const SRC: &'static str = r#"
        <!I18N>
        <view wx:if="{{item.status === 2}}">一些文字</view>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><block wx:if=\"{{item.status===2}}\"><view>Some words</view></block></block><block wx:elif=\"{{locale===\"ja\"}}\"><block wx:if=\"{{item.status===2}}\"><view>いくつかのテキスト</view></block></block><block wx:else><block wx:if=\"{{item.status===2}}\"><view>一些文字</view></block></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

#[test]
fn for_block() {
    const SRC: &'static str = r#"
        <!I18N>
        <view wx:for="{{ arr }}">一些文字</view>
    "#;
    const OUT: &'static str = "<block wx:if=\"{{locale===\"en-us\"}}\"><block wx:for=\"{{arr}}\"><view>Some words</view></block></block><block wx:elif=\"{{locale===\"ja\"}}\"><block wx:for=\"{{arr}}\"><view>いくつかのテキスト</view></block></block><block wx:else><block wx:for=\"{{arr}}\"><view>一些文字</view></block></block>";
    let out = compile("TEST", SRC, TRANSLATE_FILE, &[]).unwrap();
    assert_eq!(out.output, OUT);
}

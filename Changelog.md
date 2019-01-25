
# Table of Contents

1.  [<span class="timestamp-wrapper"><span class="timestamp">[2019-01-25 Fri] </span></span> v0.0.16](#orgf1b00c8)
2.  [<span class="timestamp-wrapper"><span class="timestamp">[2018-11-11 Sun] </span></span> v0.0.15](#org43301e3)
3.  [<span class="timestamp-wrapper"><span class="timestamp">[2018-10-31 Wed] </span></span> v0.0.14](#org26c45e8)
4.  [<span class="timestamp-wrapper"><span class="timestamp">[2018-10-07 Sun] </span></span> v0.0.13](#org1d951b9)
5.  [<span class="timestamp-wrapper"><span class="timestamp">[2018-10-06 Sat] </span></span> v0.0.12](#org4b27433)
6.  [<span class="timestamp-wrapper"><span class="timestamp">[2018-09-29 Sat] </span></span> v0.0.11](#org81b95d5)
7.  [<span class="timestamp-wrapper"><span class="timestamp">[2018-06-08 Fri] </span></span> v0.0.5](#org73a9cba)


<a id="orgf1b00c8"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2019-01-25 Fri] </span></span> v0.0.16

-   added golden section search algorithm
-   improved BlackBox model docs


<a id="org43301e3"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2018-11-11 Sun] </span></span> v0.0.15

-   更新linefeed至新版, 解除对ncurses系统库的依赖.
-   解决bbmrun命令中, lbfgs返回错误代码的问题
-   解决bbmrun命令无法输出优化后的结构


<a id="org26c45e8"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2018-10-31 Wed] </span></span> v0.0.14

-   将parser部分代码独立为一个新的crate: textparser
-   添加CG优化算法


<a id="org1d951b9"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2018-10-07 Sun] </span></span> v0.0.13

-   改进apps/optimization相关接口设计.
-   添加nebrun子程序, 用于NEB优化.


<a id="org4b27433"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2018-10-06 Sat] </span></span> v0.0.12

-   models模拟中添加BlackBox模型, 方便调用任意外部模型.
-   添加相关命令行程序: bbmrun


<a id="org81b95d5"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2018-09-29 Sat] </span></span> v0.0.11

-   runner添加\`-o\`选项, 保存计算后的结构
-   计算后的能量保存为分子的title/name


<a id="org73a9cba"></a>

# <span class="timestamp-wrapper"><span class="timestamp">[2018-06-08 Fri] </span></span> v0.0.5

新增format命令, 可以根据自己的需求设计输出模板, 比如gulp, siesta等, gosh可以根据模板定义, 替换原子坐标等分子信息, 自动生成对应的输入文件.


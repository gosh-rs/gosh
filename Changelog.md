
# <span class="timestamp-wrapper"><span class="timestamp">[2019-02-08 Fri] </span></span> v0.0.17

-   use fixed crate versions as dependencies
-   remove neb mod (moved to nebrun crate)


# <span class="timestamp-wrapper"><span class="timestamp">[2019-01-25 Fri] </span></span> v0.0.16

-   added golden section search algorithm
-   improved BlackBox model docs


# <span class="timestamp-wrapper"><span class="timestamp">[2018-11-11 Sun] </span></span> v0.0.15

-   更新linefeed至新版, 解除对ncurses系统库的依赖.
-   解决bbmrun命令中, lbfgs返回错误代码的问题
-   解决bbmrun命令无法输出优化后的结构


# <span class="timestamp-wrapper"><span class="timestamp">[2018-10-31 Wed] </span></span> v0.0.14

-   将parser部分代码独立为一个新的crate: textparser
-   添加CG优化算法


# <span class="timestamp-wrapper"><span class="timestamp">[2018-10-07 Sun] </span></span> v0.0.13

-   改进apps/optimization相关接口设计.
-   添加nebrun子程序, 用于NEB优化.


# <span class="timestamp-wrapper"><span class="timestamp">[2018-10-06 Sat] </span></span> v0.0.12

-   models模拟中添加BlackBox模型, 方便调用任意外部模型.
-   添加相关命令行程序: bbmrun


# <span class="timestamp-wrapper"><span class="timestamp">[2018-09-29 Sat] </span></span> v0.0.11

-   runner添加\`-o\`选项, 保存计算后的结构
-   计算后的能量保存为分子的title/name


# <span class="timestamp-wrapper"><span class="timestamp">[2018-06-08 Fri] </span></span> v0.0.5

新增format命令, 可以根据自己的需求设计输出模板, 比如gulp, siesta等, gosh可以根据模板定义, 替换原子坐标等分子信息, 自动生成对应的输入文件.

